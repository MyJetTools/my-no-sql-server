use core::str;
use std::{collections::BTreeMap, sync::Arc, time::Duration};

use my_no_sql_sdk::core::db_json_entity::DbJsonEntity;
use my_no_sql_server_core::rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    app::AppContext, db_sync::EventSource,
    scripts::serializers::table_attrs::TableMetadataFileContract, zip::ZipReader,
};

use super::RestoreFileName;

#[derive(Debug)]
pub enum BackupError {
    TableNotFoundInBackupFile,
    #[allow(dead_code)]
    InvalidFileName(String),
    #[allow(dead_code)]
    ZipArchiveError(String),
    #[allow(dead_code)]
    TableNotFoundToRestoreBackupAndNoMetadataFound(String),
    #[allow(dead_code)]
    InvalidFileContent {
        file_name: String,
        partition_key: String,
        err: String,
    },
}

pub async fn restore(
    app: &Arc<AppContext>,
    backup_content: Vec<u8>,
    table_name: Option<&str>,
    clean_table: bool,
) -> Result<(), BackupError> {
    let mut zip_reader = ZipReader::new(backup_content);

    let mut partitions: BTreeMap<String, Vec<RestoreFileName>> = BTreeMap::new();

    for file_name in zip_reader.get_file_names() {
        let file_name =
            RestoreFileName::new(file_name).map_err(|err| BackupError::InvalidFileName(err))?;

        match partitions.get_mut(&file_name.table_name) {
            Some(by_table) => {
                if file_name.file_type.is_metadata() {
                    by_table.insert(0, file_name);
                } else {
                    by_table.push(file_name)
                }
            }
            None => {
                partitions.insert(file_name.table_name.to_string(), vec![file_name]);
            }
        }
    }

    if partitions.is_empty() {
        return Err(BackupError::TableNotFoundInBackupFile);
    }

    match table_name {
        Some(table_name) => match partitions.remove(table_name) {
            Some(files) => {
                restore_to_db(&app, table_name, files, &mut zip_reader, clean_table).await?;
            }
            None => {
                return Err(BackupError::TableNotFoundInBackupFile);
            }
        },
        None => {
            for (table_name, files) in partitions {
                restore_to_db(
                    &app,
                    table_name.as_str(),
                    files,
                    &mut zip_reader,
                    clean_table,
                )
                .await?;
            }
        }
    }
    Ok(())
}

async fn restore_to_db(
    app: &Arc<AppContext>,
    table_name: &str,
    mut files: Vec<RestoreFileName>,
    zip: &mut ZipReader,
    clean_table: bool,
) -> Result<(), BackupError> {
    let persist_moment = DateTimeAsMicroseconds::now().add(Duration::from_secs(5));
    let db_table = if files.get(0).unwrap().file_type.is_metadata() {
        let metadata_file = files.remove(0);

        let content = zip
            .get_content_as_vec(&metadata_file.file_name)
            .map_err(|err| BackupError::ZipArchiveError(format!("{:?}", err)))?;

        println!("Content: '{}'", str::from_utf8(content.as_slice()).unwrap());

        let table = TableMetadataFileContract::parse(content.as_slice());

        let db_table = crate::db_operations::write::table::create_if_not_exist(
            app,
            table_name,
            table.persist,
            table.max_partitions_amount,
            table.max_rows_per_partition_amount,
            EventSource::Backup,
            persist_moment,
        )
        .await
        .unwrap();
        db_table
    } else {
        let db_table = app.db.get_table(table_name).await;

        if db_table.is_none() {
            return Err(BackupError::TableNotFoundToRestoreBackupAndNoMetadataFound(
                table_name.to_string(),
            ));
        }

        db_table.unwrap()
    };

    if clean_table {
        crate::db_operations::write::clean_table(
            &app,
            &db_table,
            EventSource::Backup,
            persist_moment,
        )
        .await
        .unwrap();
    }

    for partition_file in files {
        let partition_key = partition_file.file_type.unwrap_as_partition_key();

        let content = zip
            .get_content_as_vec(&partition_file.file_name)
            .map_err(|err| BackupError::ZipArchiveError(format!("{:?}", err)))?;

        let db_rows = DbJsonEntity::restore_as_vec(content.as_slice()).map_err(|itm| {
            BackupError::InvalidFileContent {
                file_name: partition_file.file_name,
                partition_key: partition_key.to_string(),
                err: format!("{:?}", itm),
            }
        })?;

        crate::db_operations::write::clean_partition_and_bulk_insert(
            app,
            &db_table,
            partition_key.to_string(),
            vec![(partition_key, db_rows)],
            EventSource::Backup,
            persist_moment,
            DateTimeAsMicroseconds::now(),
        )
        .await
        .unwrap();
    }

    Ok(())
}
