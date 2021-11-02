use std::sync::Arc;

use my_azure_storage_sdk::AzureConnection;
use rust_extensions::{date_time::DateTimeAsMicroseconds, StopWatch};

use crate::{app::AppContext, db::DbTable};

pub async fn init_tables(app: &AppContext, connection: &AzureConnection) {
    let tables = crate::blob_operations::table::get_list(connection)
        .await
        .unwrap();

    for table_name in &tables {
        app.logs
            .add_info(
                Some(table_name.to_string()),
                crate::app::logs::SystemProcess::Init,
                "init_tables".to_string(),
                format!("Initializing table {}", table_name),
            )
            .await;
        let mut sw = StopWatch::new();
        sw.start();
        let table_data = crate::blob_operations::table::load(app, &connection, table_name)
            .await
            .unwrap();

        let now = DateTimeAsMicroseconds::now();

        let db_table = DbTable::new(table_name.to_string(), table_data, now);

        let mut tables_write_access = app.db.tables.write().await;

        tables_write_access.insert(table_name.to_string(), Arc::new(db_table));

        sw.pause();

        app.logs
            .add_info(
                Some(table_name.to_string()),
                crate::app::logs::SystemProcess::Init,
                "init_tables".to_string(),
                format!("Table {} is initialized in {:?}", table_name, sw.duration()),
            )
            .await;
    }

    app.states.set_initialized();

    app.logs
        .add_info(
            None,
            crate::app::logs::SystemProcess::Init,
            "init_tables".to_string(),
            "All tables initialized".to_string(),
        )
        .await;
}
