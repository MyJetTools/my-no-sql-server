use my_azure_storage_sdk::AzureConnection;

use crate::{app::AppContext, db::DbTable};

pub async fn execute(app: &AppContext, db_table: &DbTable, azure_connection: &AzureConnection) {
    let attr = db_table.get_attributes().await;
    crate::blob_operations::save_table_attributes::with_retries(
        app,
        azure_connection,
        db_table.name.as_str(),
        &attr,
    )
    .await;
}
