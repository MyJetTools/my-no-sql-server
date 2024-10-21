use std::sync::Arc;

use my_logger::LogEventCtx;
use my_no_sql_sdk::core::{
    db::{DbPartition, DbTable, DbTableAttributes},
    db_json_entity::DbJsonEntity,
    rust_extensions::StopWatch,
};
use my_no_sql_server_core::DbTableWrapper;

use crate::{
    app::AppContext, persist_io::TableFile,
    persist_operations::serializers::TableMetadataFileContract,
};

pub async fn load_tables(app: Arc<AppContext>) {
    let mut sw = StopWatch::new();
    sw.start();
    if let Some(url) = app.settings.get_init_from_other_server_url() {
        super::from_other_instance::init_from_another_instance(&app, url).await;
    } else {
        if app.persist_io.is_sqlite() {
            init_as_sql_lite(&app).await;
        } else {
            init_from_storage(&app).await;
        }
    }

    app.states.set_initialized();

    app.init_state.dispose().await;

    sw.pause();

    my_logger::LOGGER.write_info(
        "init_tables".to_string(),
        format!("All tables initialized in {:?}", sw.duration()),
        LogEventCtx::new(),
    );
}

async fn init_as_sql_lite(app: &Arc<AppContext>) {
    let table_names = app.persist_io.get_list_of_tables().await;

    println!("Got tables {:?}", table_names.len());

    let mut loaded = 0;
    for table_name in table_names {
        if loaded % 10 == 0 {
            println!("Loaded {} tables", loaded);
        }
        loaded += 1;

        let db_table = app
            .persist_io
            .get_table_files_as_list(table_name.as_str())
            .await;

        if db_table.is_none() {
            panic!("Table {} is not found. Something went wrong", table_name);
        }

        let mut table_attributes = None;

        let mut partitions = Vec::new();

        for (file_name, file_content) in db_table.unwrap() {
            let table_file = TableFile::from_file_name(file_name.as_str()).unwrap();

            match table_file {
                TableFile::TableAttributes => {
                    let metadata_file = TableMetadataFileContract::parse(file_content.as_slice());

                    let attr: DbTableAttributes = metadata_file.into();
                    table_attributes = Some(attr);
                }
                TableFile::DbPartition(partition_key) => {
                    let db_rows = DbJsonEntity::restore_as_vec(file_content.as_slice()).unwrap();
                    let mut restored_partition = DbPartition::new(partition_key);
                    restored_partition.insert_or_replace_rows_bulk(db_rows.as_slice());
                    partitions.push(restored_partition);
                }
            }
        }

        let mut db_table = DbTable::new(table_name, table_attributes.unwrap_or_default());

        for db_partition in partitions {
            db_table.init_partition(db_partition);
        }

        let db_table = DbTableWrapper::new(db_table);

        let mut table_access = app.db.tables.write().await;
        table_access.insert(db_table.name.to_string(), db_table);

        // crate::db_operations::write::table::init(app.as_ref(), db_table).await;
    }
}

async fn init_from_storage(app: &Arc<AppContext>) {
    let table_names = app.persist_io.get_list_of_tables().await;

    app.init_state.init_table_names(table_names.clone()).await;

    tokio::spawn(super::table_list_of_files_loader(app.clone(), table_names));

    let mut threads = Vec::new();
    for _ in 0..app.settings.init_threads_amount {
        threads.push(tokio::spawn(super::load_table_files::spawn(app.clone())));
    }

    for thread in threads {
        thread.await.unwrap();
    }

    while let Some(db_table) = app.init_state.get_table_data_result().await {
        crate::db_operations::write::table::init(app.as_ref(), db_table).await;
    }
}
