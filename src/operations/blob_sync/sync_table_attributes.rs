use my_azure_storage_sdk::AzureConnectionWithTelemetry;

use crate::{app::AppContext, db::DbTable};

use my_app_insights::AppInsightsTelemetry;

pub async fn execute(
    app: &AppContext,
    db_table: &DbTable,
    azure_connection: &AzureConnectionWithTelemetry<AppInsightsTelemetry>,
) {
    let attr = db_table.get_attributes().await;
    crate::blob_operations::save_table_attributes::with_retries(
        app,
        azure_connection,
        db_table.name.as_str(),
        &attr,
    )
    .await;
}
