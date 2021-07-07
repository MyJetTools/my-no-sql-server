use std::{sync::Arc, time::Duration};

use my_azure_storage_sdk::AzureConnection;

use crate::app::{logs::SystemProcess, AppServices};

pub async fn start(app: Arc<AppServices>, azure_connection: Arc<AzureConnection>) {
    app.logs
        .add_info(
            None,
            SystemProcess::System,
            "Timer blob persistence initialization".to_string(),
            "Started".to_string(),
        )
        .await;
    let delay = Duration::from_secs(5);

    loop {
        tokio::time::sleep(delay).await;
        iterate(app.as_ref(), azure_connection.as_ref()).await;
    }
}

async fn iterate(app: &AppServices, azure_connection: &AzureConnection) {
    while let Some(transaction_event) = app.queue_to_persist.dequeue().await {
        match transaction_event.as_ref() {
            crate::db_transactions::TransactionEvent::UpdateTableAttributes {
                table,
                attr: _,
                table_is_just_created,
                persist: _,
                max_partitions_amount: _,
            } => {
                if *table_is_just_created {
                    crate::persistence::data_synchronizer::update_table(
                        app,
                        table.name.as_str(),
                        azure_connection,
                    )
                    .await;
                } else {
                    crate::persistence::blob::save_table_attributes::with_retries(
                        app,
                        azure_connection,
                        table.name.as_str(),
                        &table.get_attributes().await,
                    )
                    .await;
                }
            }
            crate::db_transactions::TransactionEvent::InitTable {
                table,
                attr: _,
                partitions: _,
            } => {
                crate::persistence::data_synchronizer::update_table(
                    app,
                    table.name.as_str(),
                    azure_connection,
                )
                .await;
            }
            crate::db_transactions::TransactionEvent::DeleteTable { table, attr: _ } => {
                crate::persistence::data_synchronizer::update_table(
                    app,
                    table.name.as_str(),
                    azure_connection,
                )
                .await;
            }
            crate::db_transactions::TransactionEvent::CleanTable { table, attr: _ } => {
                crate::persistence::data_synchronizer::update_table(
                    app,
                    table.name.as_str(),
                    azure_connection,
                )
                .await;
            }
            crate::db_transactions::TransactionEvent::DeletePartitions {
                table,
                partitions,
                attr: _,
            } => {
                crate::persistence::data_synchronizer::update_partitions(
                    app,
                    table.name.as_str(),
                    partitions.as_slice(),
                    azure_connection,
                )
                .await;
            }
            crate::db_transactions::TransactionEvent::UpdateRow {
                table,
                attr: _,
                partition_key,
                row: _,
            } => {
                let partitions = [partition_key.to_string()];
                crate::persistence::data_synchronizer::update_partitions(
                    app,
                    table.name.as_str(),
                    &partitions[..],
                    azure_connection,
                )
                .await;
            }
            crate::db_transactions::TransactionEvent::UpdateRows {
                table,
                attr: _,
                rows_by_partition,
            } => {
                let partition_keys: Vec<String> = rows_by_partition
                    .keys()
                    .into_iter()
                    .map(|itm| itm.to_string())
                    .collect();

                crate::persistence::data_synchronizer::update_partitions(
                    app,
                    table.name.as_str(),
                    &partition_keys[..],
                    azure_connection,
                )
                .await;
            }
            crate::db_transactions::TransactionEvent::DeleteRows {
                table,
                attr: _,
                rows,
            } => {
                let partition_keys: Vec<String> =
                    rows.keys().into_iter().map(|itm| itm.to_string()).collect();

                crate::persistence::data_synchronizer::update_partitions(
                    app,
                    table.name.as_str(),
                    &partition_keys[..],
                    azure_connection,
                )
                .await;
            }
        };
    }
}
