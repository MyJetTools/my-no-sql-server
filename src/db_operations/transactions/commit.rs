use std::collections::HashMap;

use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    app::AppContext, db_sync::EventSource, db_transactions::steps::TransactionalOperationStep,
};

use super::TransactionOperationError;

pub async fn commit(
    app: &AppContext,
    transaction_id: &str,
    event_src: EventSource,
    persist_moment: DateTimeAsMicroseconds,
    now: DateTimeAsMicroseconds,
) -> Result<(), TransactionOperationError> {
    let transaction = app.active_transactions.remove(transaction_id).await;

    if transaction.is_none() {
        return Err(TransactionOperationError::TransactionNotFound {
            id: transaction_id.to_string(),
        });
    }

    let transaction = transaction.unwrap();

    let mut tables = HashMap::new();

    for table_name in transaction.operations.keys() {
        let db_table = crate::db_operations::read::table::get(app, table_name).await?;
        tables.insert(table_name.to_string(), db_table);
    }

    for (_, mut events) in transaction.operations {
        for event in events.drain(..) {
            match event {
                TransactionalOperationStep::CleanTable { table_name } => {
                    let db_table = tables.get(table_name.as_str()).unwrap();
                    crate::db_operations::write::clean_table::execute(
                        app,
                        &db_table,
                        event_src.clone(),
                        persist_moment,
                    )
                    .await?;
                }
                TransactionalOperationStep::DeletePartitions {
                    table_name,
                    partition_keys,
                } => {
                    let db_table = tables.get(table_name.as_str()).unwrap();
                    crate::db_operations::write::delete_partitions(
                        app,
                        &db_table,
                        partition_keys.into_iter(),
                        event_src.clone(),
                        persist_moment,
                        now,
                    )
                    .await?;
                }
                TransactionalOperationStep::DeleteRows {
                    table_name,
                    partition_key,
                    row_keys,
                } => {
                    let db_table = tables.get(table_name.as_str()).unwrap();
                    let mut rows_to_delete = HashMap::new();
                    rows_to_delete.insert(partition_key, row_keys);
                    crate::db_operations::write::bulk_delete(
                        app,
                        db_table.as_ref(),
                        rows_to_delete,
                        event_src.clone(),
                        persist_moment,
                        now,
                    )
                    .await?;
                }
                TransactionalOperationStep::UpdateRows(state) => {
                    let db_table = tables.get(state.table_name.as_str()).unwrap();
                    crate::db_operations::write::bulk_insert_or_update::execute(
                        app,
                        &db_table,
                        state.rows_by_partition,
                        event_src.clone(),
                        persist_moment,
                        now,
                    )
                    .await?;
                }
            }
        }
    }

    Ok(())
}
