use std::sync::Arc;

use my_azure_storage_sdk::AzureStorageConnection;
use rust_extensions::StopWatch;

use crate::app::AppContext;

pub async fn init_tables(
    app: Arc<AppContext>,
    connection: Arc<AzureStorageConnection>,
    init_threads_amount: usize,
) {
    tokio::spawn(init_tables_spawned(app, connection, init_threads_amount));
}

async fn init_tables_spawned(
    app: Arc<AppContext>,
    connection: Arc<AzureStorageConnection>,
    init_threads_amount: usize,
) {
    let tables = crate::blob_operations::table::get_list(connection.as_ref())
        .await
        .unwrap();

    for table_name in tables {
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
        let (table_data, attr) = crate::blob_operations::table::load(
            connection.clone(),
            &table_name,
            init_threads_amount,
        )
        .await
        .unwrap();

        crate::db_operations::write::table::init(app.as_ref(), table_data, attr).await;

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
