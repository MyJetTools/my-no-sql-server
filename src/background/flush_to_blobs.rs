use std::{sync::Arc, time::Duration};

use my_app_insights::AppInsightsTelemetry;
use my_azure_storage_sdk::AzureStorageConnectionWithTelemetry;

use crate::{app::AppContext, persistence::PersistResult};

pub async fn start(
    app: Arc<AppContext>,
    azure_connection: Arc<AzureStorageConnectionWithTelemetry<AppInsightsTelemetry>>,
) {
    let one_sec = Duration::from_secs(1);
    while !app.states.is_initialized() {
        tokio::time::sleep(one_sec).await;
    }

    println!("Persistence loop is started");

    loop {
        let result = tokio::spawn(iteration(app.clone(), azure_connection.clone())).await;

        if let Err(err) = result {
            println!("flush_to_blobs_err: {:?}", err);
        }

        tokio::time::sleep(one_sec).await;
    }
}

async fn iteration(
    app: Arc<AppContext>,
    azure_connection: Arc<AzureStorageConnectionWithTelemetry<AppInsightsTelemetry>>,
) {
    let is_shutting_down = app.states.is_shutting_down();

    let tables = app.db.get_tables().await;

    for db_table in tables {
        if let Some(persist_result) = db_table.get_what_to_persist(is_shutting_down).await {
            match persist_result {
                PersistResult::PersistAttrs => {
                    crate::operations::blob_sync::sync_table_attributes::execute(
                        app.as_ref(),
                        db_table.as_ref(),
                        azure_connection.as_ref(),
                    )
                    .await;
                }
                PersistResult::PersistTable => {
                    crate::operations::blob_sync::sync_table::sync_everything(
                        app.as_ref(),
                        db_table.as_ref(),
                        azure_connection.as_ref(),
                    )
                    .await;
                }
                PersistResult::PersistPartition(partition_key) => {
                    crate::operations::blob_sync::sync_partition::execute(
                        app.as_ref(),
                        db_table.as_ref(),
                        azure_connection.as_ref(),
                        partition_key.as_str(),
                    )
                    .await;
                }
            }
        }
    }
}
