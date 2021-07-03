use std::sync::Arc;

use my_azure_storage_sdk::AzureConnection;

use crate::{app::AppServices, date_time::MyDateTime, db::DbTable};

pub async fn init_tables(app: &AppServices) {
    let connection = AzureConnection::from_conn_string(app.settings.persistence_dest.as_str());

    let tables = super::blob_repo::get_tables(&connection).await.unwrap();

    for table_name in &tables {
        println!("Loading table: {}", table_name);
        let table_data = super::blob_repo::load_table(&connection, table_name)
            .await
            .unwrap();

        let now = MyDateTime::utc_now();

        let db_table = DbTable::new(table_name.to_string(), table_data, now);

        let mut tables_write_access = app.db.tables.write().await;

        tables_write_access.insert(table_name.to_string(), Arc::new(db_table));
    }

    println!("All Tables are initialized");
}
