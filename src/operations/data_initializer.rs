use std::sync::Arc;

use rust_extensions::{date_time::DateTimeAsMicroseconds, StopWatch};

use crate::{
    app::AppContext,
    db::{DbTableAttributesSnapshot, DbTableData},
    persist_io::TableLoadItem,
};

use super::tables_to_initialize::TablesToInitialize;

pub async fn init_tables(app: Arc<AppContext>, init_threads_amount: usize) {
    tokio::spawn(init_tables_spawned(app, init_threads_amount));
}

//TODO - Make it multithreaded
async fn init_tables_spawned(app: Arc<AppContext>, init_threads_amount: usize) {
    let tables = app.persist_io.get_list_of_tables().await;

    /*
       let tables: Vec<String> = tables
           .iter()
           .filter(|itm| *itm == "tokens-manager-short-sessions")
           .map(|itm| itm.to_string())
           .collect();
    */

    let tables = Arc::new(TablesToInitialize::new(tables));

    let mut sw = StopWatch::new();
    sw.start();
    let mut threads = Vec::new();
    for _ in 0..init_threads_amount {
        let task_join = tokio::spawn(load_tables_thread(tables.clone(), app.clone()));
        threads.push(task_join);
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

async fn load_tables_thread(tables: Arc<TablesToInitialize>, app: Arc<AppContext>) {
    while let Some(table_name) = tables.get().await {
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

        let mut item = app.persist_io.start_loading_table(&table_name).await;

        while let Some(table_load_item) = item {
            //    println!("Loading partition: {}", table_load_item.as_str());
            match table_load_item {
                TableLoadItem::TableAttributes(attrs) => {
                    db_table_attirbutes = Some(attrs);
                }
                TableLoadItem::DbPartition {
                    partition_key,
                    db_partition,
                } => db_table_data.init_partition(partition_key, db_partition),
            }

            item = app.persist_io.continue_loading_table(&table_name).await;
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
}
