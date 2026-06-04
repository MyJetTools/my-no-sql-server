use core::str;
use std::{collections::BTreeMap, sync::Arc, time::Duration};

use my_no_sql_sdk::core::db_json_entity::DbJsonEntity;
use my_no_sql_sdk::server::rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    app::AppContext,
    db_sync::{states::InitTableEventSyncData, EventSource, SyncEvent},
    scripts::serializers::table_attrs::TableMetadataFileContract,
    zip::ZipReader,
};

use super::RestoreFileName;

#[derive(Debug)]
pub enum BackupError {
    TableNotFoundInBackupFile,
    #[allow(dead_code)]
    InvalidFileName(String),
    #[allow(dead_code)]
    FileReadError(String),
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

impl BackupError {
    pub fn into_message(self) -> String {
        match self {
            BackupError::TableNotFoundInBackupFile => {
                "Table not found in the backup file".to_string()
            }
            BackupError::InvalidFileName(err) => format!("Invalid file name: {}", err),
            BackupError::FileReadError(err) => err,
            BackupError::ZipArchiveError(err) => format!("Zip archive error: {}", err),
            BackupError::TableNotFoundToRestoreBackupAndNoMetadataFound(table) => format!(
                "Table '{}' is not present on the server and the snapshot has no metadata to recreate it",
                table
            ),
            BackupError::InvalidFileContent {
                file_name,
                partition_key,
                err,
            } => format!(
                "Invalid content in '{}' (partition '{}'): {}",
                file_name, partition_key, err
            ),
        }
    }
}

/// Reads a snapshot file by name from the server's backup folder and restores
/// it. `table_name == None` restores every table found in the snapshot.
pub async fn restore_from_file(
    app: &Arc<AppContext>,
    file_name: &str,
    table_name: Option<&str>,
    clean_table: bool,
) -> Result<(), BackupError> {
    let full_path = super::utils::compile_backup_file(app, file_name);
    let backup_content = tokio::fs::read(full_path.as_str()).await.map_err(|err| {
        BackupError::FileReadError(format!("Error loading file '{}': {}", file_name, err))
    })?;

    restore(app, backup_content, table_name, clean_table).await
}

pub async fn restore(
    app: &Arc<AppContext>,
    backup_content: Vec<u8>,
    table_name: Option<&str>,
    clean_table: bool,
) -> Result<(), BackupError> {
    let mut zip_reader = ZipReader::new(backup_content);

    let mut partitions: BTreeMap<String, Vec<RestoreFileName>> = BTreeMap::new();

    for file_name_str in zip_reader.get_file_names() {
        let file_name =
            RestoreFileName::new(file_name_str).map_err(|err| BackupError::InvalidFileName(err))?;

        if file_name.is_none() {
            println!("Skipping  file [{}]", file_name_str);
            continue;
        }

        let file_name = file_name.unwrap();
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

/// Restores a single partition of a table from a snapshot file. The table must
/// already exist on the server (restore the whole table first if it does not).
/// The partition content is replaced with the rows stored in the snapshot.
pub async fn restore_partition(
    app: &Arc<AppContext>,
    file_name: &str,
    table_name: &str,
    partition_key: &str,
) -> Result<(), String> {
    let content =
        super::read_snapshot_partition_rows(app, file_name, table_name, partition_key)
            .await
            .map_err(|err| err.into_message())?;

    let db_rows = DbJsonEntity::restore_as_vec(content.as_slice())
        .map_err(|err| format!("Invalid partition content: {:?}", err))?;

    let db_table = app.db.get_table(table_name).ok_or_else(|| {
        format!(
            "Table '{}' does not exist on the server. Restore the whole table first.",
            table_name
        )
    })?;

    let persist_moment = DateTimeAsMicroseconds::now().add(Duration::from_secs(5));

    crate::db_operations::write::clean_partition_and_bulk_insert(
        app,
        &db_table,
        partition_key.to_string(),
        vec![(partition_key.to_string(), db_rows)],
        EventSource::Backup,
        persist_moment,
        DateTimeAsMicroseconds::now(),
    )
    .await
    .map_err(|err| format!("{:?}", err))?;

    // Emit a full table-init so subscribers re-initialize after the restore,
    // same as the whole-table restore path (see `restore_to_db`).
    {
        let table_data = db_table.data.read();
        let sync_data = InitTableEventSyncData::new(&table_data, EventSource::Backup);
        crate::operations::sync::dispatch(app, SyncEvent::InitTable(sync_data));
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
        let db_table = app.db.get_table(table_name);

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

    // The per-partition writes above only emit InitPartitions events, so a
    // reader subscribed to the whole table never receives a fresh full
    // snapshot after a restore. Dispatch a table-init with the now-restored
    // state so every subscriber re-initializes its local copy of the table.
    {
        let table_data = db_table.data.read();
        let sync_data = InitTableEventSyncData::new(&table_data, EventSource::Backup);
        crate::operations::sync::dispatch(app, SyncEvent::InitTable(sync_data));
    }

    Ok(())
}
