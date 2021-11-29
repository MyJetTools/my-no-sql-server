use std::{sync::Arc, time::Duration};

use my_app_insights::AppInsightsTelemetry;
use my_azure_storage_sdk::AzureStorageConnectionWithTelemetry;
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{app::AppContext, persistence::updates_to_persist::TableUpdatesState};

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
    let now = DateTimeAsMicroseconds::now();

    let is_shutting_down = app.states.is_shutting_down();

    while let Some(persist_event) = app
        .updates_to_persist_by_table
        .get_next_sync_event(now, is_shutting_down)
        .await
    {
        let get_table_result = app.db.get_table(persist_event.table_name.as_str()).await;

        match get_table_result {
            Some(db_table) => match persist_event.state {
                TableUpdatesState::PartitionsAreUpdated(data) => {
                    if data.common_state.sync_table_attrs {
                        crate::operations::blob_sync::sync_table_attributes::execute(
                            app.as_ref(),
                            db_table.as_ref(),
                            azure_connection.as_ref(),
                        )
                        .await
                    }

                    for partition_key in data.partitions.keys() {
                        crate::operations::blob_sync::sync_partition::execute(
                            app.as_ref(),
                            db_table.as_ref(),
                            azure_connection.as_ref(),
                            partition_key,
                        )
                        .await;
                    }
                }
                TableUpdatesState::TableIsUpdated(data) => {
                    if data.common_state.sync_table_attrs {
                        crate::operations::blob_sync::sync_table_attributes::execute(
                            app.as_ref(),
                            db_table.as_ref(),
                            azure_connection.as_ref(),
                        )
                        .await;
                    }

                    crate::operations::blob_sync::sync_table::sync_everythin(
                        app.as_ref(),
                        db_table.as_ref(),
                        azure_connection.as_ref(),
                    )
                    .await;
                }
            },
            None => {
                crate::blob_operations::delete_table::with_retries(
                    app.as_ref(),
                    azure_connection.as_ref(),
                    persist_event.table_name.as_str(),
                )
                .await;
            }
        }
    }
}
