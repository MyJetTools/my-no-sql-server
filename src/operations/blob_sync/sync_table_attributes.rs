use my_azure_storage_sdk::AzureStorageConnection;

use crate::{app::AppContext, db::DbTable};

pub async fn execute(
    app: &AppContext,
    db_table: &DbTable,
    azure_connection: &AzureStorageConnection,
) {
    let attr = db_table.attributes.get_snapshot();
    crate::blob_operations::save_table_attributes::with_retries(
        app,
        azure_connection,
        db_table.name.as_str(),
        &attr,
    )
    .await;
}
