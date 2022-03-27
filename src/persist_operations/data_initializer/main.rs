use std::sync::Arc;

use crate::{
    app::AppContext,
    db::{DbTableAttributesSnapshot, DbTableData},
    persist_io::TableFile,
    persist_operations::serializers::TableMetadataFileContract,
};
use rust_extensions::{date_time::DateTimeAsMicroseconds, StopWatch};

use super::{super::serializers, TablesToLoad};
use super::{
    loaded_table::LoadedTable, table_files_to_load::TableFilesToLoad,
    table_load_item::TableLoadItem,
};

pub async fn init_tables(app: Arc<AppContext>) {
    tokio::spawn(init_tables_spawned(app));
}

async fn init_tables_spawned(app: Arc<AppContext>) {
    let tables = app.persist_io.get_list_of_tables().await;

    app.init_state
        .init(tables.as_ref(), app.logs.as_ref())
        .await;

    let tables = Arc::new(TablesToLoad::new(tables));

    let mut sw = StopWatch::new();
    sw.start();

    let mut threads = Vec::new();
    for _ in 0..app.settings.init_tabes_threads_amount {
        threads.push(tokio::spawn(load_tables(app.clone(), tables.clone())));
    }

    for thread in threads {
        thread.await.unwrap();
    }

    app.states.set_initialized();

    sw.pause();

    app.logs.add_info(
        None,
        crate::app::logs::SystemProcess::Init,
        "init_tables".to_string(),
        format!("All tables initialized in {:?}", sw.duration()),
    );
}

async fn load_tables(app: Arc<AppContext>, tables: Arc<TablesToLoad>) {
    while let Some(table_name) = tables.get().await {
        load_table(table_name, &app).await;
    }
}

async fn load_table(table_name: String, app: &Arc<AppContext>) {
    app.logs.add_info(
        Some(table_name.to_string()),
        crate::app::logs::SystemProcess::Init,
        "init_tables".to_string(),
        format!("Initializing table {}", table_name),
    );
    let mut sw = StopWatch::new();
    sw.start();

    let now = DateTimeAsMicroseconds::now();

    let mut db_table_data = DbTableData::new(table_name.to_string(), now);

    let mut db_table_attirbutes: Option<DbTableAttributesSnapshot> = None;

    let table_items = load_partitions(table_name.as_str(), app.clone()).await;

    for table_item in table_items.get().await {
        match table_item {
            TableLoadItem::TableAttributes(attr) => {
                db_table_attirbutes = Some(attr);
            }
            TableLoadItem::DbPartition {
                partition_key,
                db_partition,
            } => {
                db_table_data.partitions.insert(partition_key, db_partition);
            }
        }
    }

    let attr = if let Some(attr) = db_table_attirbutes {
        attr
    } else {
        DbTableAttributesSnapshot::create_default()
    };

    crate::db_operations::write::table::init(app.as_ref(), db_table_data, attr).await;
    sw.pause();
    app.logs.add_info(
        Some(table_name.to_string()),
        crate::app::logs::SystemProcess::Init,
        "init_tables".to_string(),
        format!("Table {} is initialized in {:?}", table_name, sw.duration()),
    );
}

async fn load_partitions(table_name: &str, app: Arc<AppContext>) -> Arc<LoadedTable> {
    let files_to_load = app.persist_io.get_table_files(&table_name).await;

    app.init_state
        .update_partitions_amount(table_name, files_to_load.len())
        .await;

    let files_to_load = Arc::new(TableFilesToLoad::new(files_to_load));

    let loaded_table = Arc::new(LoadedTable::new());

    let mut threads = Vec::new();
    for _ in 0..app.settings.init_partitions_threads_amount {
        threads.push(tokio::spawn(load_partitions_thread(
            table_name.to_string(),
            files_to_load.clone(),
            loaded_table.clone(),
            app.clone(),
        )))
    }

    for thread in threads {
        thread.await.unwrap();
    }

    app.init_state.loaded_completed(table_name).await;

    loaded_table
}

async fn load_partitions_thread(
    table_name: String,
    partitions_to_load: Arc<TableFilesToLoad>,
    loaded_table: Arc<LoadedTable>,
    app: Arc<AppContext>,
) {
    while let Some(table_file) = partitions_to_load.get_next().await {
        let content = app
            .persist_io
            .load_table_file(table_name.as_str(), &table_file)
            .await;

        if let Some(content) = content.as_ref() {
            match get_item(&table_file, content) {
                Ok(item) => {
                    let amount = loaded_table.add(item).await;
                    app.init_state
                        .update_loaded(table_name.as_str(), amount)
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
                            table_name.as_str(),
                            table_file.get_file_name().as_str(),
                            err
                        );
                    }
                }
            }
        }
    }
}

fn get_item(table_file: &TableFile, content: &[u8]) -> Result<TableLoadItem, String> {
    match table_file {
        TableFile::TableAttributes => {
            let table_metadata = TableMetadataFileContract::parse(content);
            let result = TableLoadItem::TableAttributes(table_metadata.into());
            return Ok(result);
        }
        TableFile::DbPartition(partition_key) => {
            let db_partition = serializers::db_partition::deserialize(content)?;

            let result = TableLoadItem::DbPartition {
                partition_key: partition_key.to_string(),
                db_partition,
            };

            return Ok(result);
        }
    }
}
