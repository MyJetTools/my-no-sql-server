use std::sync::Arc;

use tokio::sync::mpsc::UnboundedReceiver;

use crate::{
    app::{logs::LogItem, AppServices},
    db_operations::read_as_json::{hash_map_to_vec, DbEntityAsJsonArray},
    db_transactions::TransactionEvent,
    json,
    utils::ItemsOrNone,
};

use super::data_reader_contract::DataReaderContract;

pub enum DataReadersCommand {
    Subscribe {
        connection_id: u64,
        table_name: String,
    },

    TransactionEvent(Arc<TransactionEvent>),
}

pub async fn start(app: Arc<AppServices>, mut receiver: UnboundedReceiver<DataReadersCommand>) {
    loop {
        let msg = receiver.recv().await;

        if msg.is_none() {
            app.logs
                .add_error(
                    None,
                    crate::app::logs::SystemProcess::System,
                    "Reading Data Readers Broadcast".to_string(),
                    "We got None message".to_string(),
                    None,
                )
                .await;
        }

        match msg.unwrap() {
            DataReadersCommand::Subscribe {
                connection_id,
                table_name,
            } => {
                let data_reader = app.data_readers.get(&connection_id).await;

                if let Some(data_reader) = data_reader {
                    data_reader.subscribe_to_table(table_name).await;
                }
            }
            DataReadersCommand::TransactionEvent(event) => {
                let result = handle_transaction_event(app.as_ref(), event).await;
                app.logs.handle_aggregated_result(result).await;
            }
        }
    }
}

async fn handle_transaction_event(
    app: &AppServices,
    event: Arc<TransactionEvent>,
) -> Result<(), ItemsOrNone<LogItem>> {
    match event.as_ref() {
        TransactionEvent::InitTable {
            table_name,
            attr: _,
            partitions,
        } => {
            app.data_readers
                .broadcast(DataReaderContract::InitTable {
                    table_name: table_name.to_string(),
                    data: hash_map_to_vec(partitions),
                })
                .await?;
        }

        TransactionEvent::UpdateRow {
            table_name,
            partition_key: _,
            attr: _,
            row,
        } => {
            app.data_readers
                .broadcast(DataReaderContract::UpdateRows {
                    table_name: table_name.to_string(),
                    data: row.as_ref().as_json_array(),
                })
                .await?;
        }

        TransactionEvent::UpdateRows {
            table_name,
            rows_by_partition,
            attr: _,
        } => {
            app.data_readers
                .broadcast(DataReaderContract::UpdateRows {
                    table_name: table_name.to_string(),
                    data: hash_map_to_vec(rows_by_partition),
                })
                .await?;
        }

        TransactionEvent::CleanTable {
            table_name,
            attr: _,
        } => {
            app.data_readers
                .broadcast(DataReaderContract::InitTable {
                    table_name: table_name.to_string(),
                    data: crate::json::consts::EMPTY_ARRAY.to_vec(),
                })
                .await?;
        }

        TransactionEvent::DeletePartitions {
            table_name,
            partitions,
            attr: _,
        } => {
            for partition_key in partitions {
                app.data_readers
                    .broadcast(DataReaderContract::InitPartition {
                        table_name: table_name.to_string(),
                        partition_key: partition_key.to_string(),
                        data: crate::json::consts::EMPTY_ARRAY.to_vec(),
                    })
                    .await?;
            }
        }

        TransactionEvent::DeleteRows {
            table_name,
            attr: _,
            rows,
        } => {
            let mut rows_to_delete = Vec::new();

            for (partition_key, db_rows) in rows {
                for db_row in db_rows {
                    rows_to_delete.push((partition_key.to_string(), db_row.to_string()));
                }
            }

            app.data_readers
                .broadcast(DataReaderContract::DeleteRows {
                    table_name: table_name.to_string(),
                    rows: rows_to_delete,
                })
                .await?;
        }
        TransactionEvent::UpdateTableAttributes {
            table_name: _,
            attr: _,
            persist: _,
            max_partitions_amount: _,
        } => {}
        TransactionEvent::DeleteTable { db_table, attr: _ } => {
            let contract = DataReaderContract::InitTable {
                table_name: db_table.name.to_string(),
                data: json::consts::EMPTY_ARRAY.to_vec(),
            };
            app.data_readers.broadcast(contract).await?;
        }
    }

    Ok(())
}
