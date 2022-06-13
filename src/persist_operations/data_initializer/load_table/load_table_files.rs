use std::{sync::Arc, time::Duration};

use crate::{
    app::AppContext,
    persist_io::TableFile,
    persist_operations::{
        data_initializer::load_tasks::{PartitionToLoad, TableToLoad},
        serializers::{self, TableMetadataFileContract},
    },
};

use super::{LoadedTable, LoadedTableItem};

pub async fn load_table_files(
    app: Arc<AppContext>,
    table_to_load: &Arc<TableToLoad>,
) -> Arc<LoadedTable> {
    let loaded_table = Arc::new(LoadedTable::new());

    loop {
        match table_to_load.get_next().await {
            Some(next_task) => match next_task {
                PartitionToLoad::Load(file_name) => {
                    load_table_file(
                        table_to_load.table_name.as_str(),
                        file_name,
                        &loaded_table,
                        &app,
                    )
                    .await;
                }
                PartitionToLoad::EndOfReading => {
                    break;
                }
            },
            None => {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
    }

    loaded_table
}

async fn load_table_file(
    table_name: &str,
    file_name: String,
    loaded_table: &Arc<LoadedTable>,
    app: &Arc<AppContext>,
) {
    let table_file = TableFile::from_file_name(file_name.as_str());

    if let Err(err) = table_file {
        app.logs.add_error(
            Some(file_name.to_string()),
            crate::app::logs::SystemProcess::Init,
            "init_tables".to_string(),
            format!("Error loading table file {}: {}", file_name, err),
            None,
        );
        return;
    }

    let table_file = table_file.unwrap();

    let content = app
        .persist_io
        .load_table_file(file_name.as_str(), &table_file)
        .await;

    if let Some(content) = content.as_ref() {
        match get_item(&table_file, content) {
            Ok(item) => {
                loaded_table.add(item).await;
                app.init_state
                    .update_file_is_loaded(file_name.as_str())
                    .await;
            }
            Err(err) => {
                if app.settings.skip_broken_partitions {
                    println!(
                        "Skipping file {}. Reason: {:?}",
                        table_file.get_file_name().as_str(),
                        err
                    );
                } else {
                    panic!(
                            "Partition is broken. Stopping initialization because of the file {}/{}. Reason: {:?}",
                            table_name,
                            table_file.get_file_name().as_str(),
                            err
                        );
                }
            }
        }
    }
}

fn get_item(table_file: &TableFile, content: &[u8]) -> Result<LoadedTableItem, String> {
    match table_file {
        TableFile::TableAttributes => {
            let table_metadata = TableMetadataFileContract::parse(content);
            let result = LoadedTableItem::TableAttributes(table_metadata.into());
            return Ok(result);
        }
        TableFile::DbPartition(partition_key) => {
            let db_partition = serializers::db_partition::deserialize(content)?;

            let result = LoadedTableItem::DbPartition {
                partition_key: partition_key.to_string(),
                db_partition,
            };

            return Ok(result);
        }
    }
}
