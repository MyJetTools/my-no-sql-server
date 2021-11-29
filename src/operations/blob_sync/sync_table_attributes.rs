use my_azure_storage_sdk::AzureStorageConnectionWithTelemetry;

use crate::{app::AppContext, db::DbTable};

use my_app_insights::AppInsightsTelemetry;

pub async fn execute(
    app: &AppContext,
    db_table: &DbTable,
    azure_connection: &AzureStorageConnectionWithTelemetry<AppInsightsTelemetry>,
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
