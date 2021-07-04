use std::sync::Arc;

use my_azure_storage_sdk::AzureConnection;

use crate::{app::AppServices, date_time::MyDateTime, db::DbTable, utils::StopWatch};

pub async fn init_tables(app: &AppServices) {
    let connection = AzureConnection::from_conn_string(app.settings.persistence_dest.as_str());

    let tables = super::blob_repo::get_tables(&connection).await.unwrap();

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
        let table_data = super::blob_repo::load_table(app, &connection, table_name)
            .await
            .unwrap();

        let now = MyDateTime::utc_now();

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

    app.logs
        .add_info(
            None,
            crate::app::logs::SystemProcess::Init,
            "init_tables".to_string(),
            "All tables initialized".to_string(),
        )
        .await;
}
