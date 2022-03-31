use std::collections::HashMap;

use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    app::AppContext, db_json_entity::JsonTimeStamp, db_sync::EventSource,
    db_transactions::steps::TransactionalOperationStep,
};

use super::TransactionOperationError;

pub async fn commit(
    app: &AppContext,
    transaction_id: &str,
    event_src: EventSource,
    now: &JsonTimeStamp,
    persist_moment: DateTimeAsMicroseconds,
) -> Result<(), TransactionOperationError> {
    let transaction = app.active_transactions.remove(transaction_id).await;

    if transaction.is_none() {
        return Err(TransactionOperationError::TransactionNotFound {
            id: transaction_id.to_string(),
        });
    }

    let mut transaction = transaction.unwrap();

    let mut tables = HashMap::new();

    for table_name in transaction.operations.keys() {
        let db_table = crate::db_operations::read::table::get(app, table_name).await?;
        tables.insert(table_name.to_string(), db_table);
    }

    for (_, mut events) in transaction.operations.drain() {
        for event in events.drain(..) {
            match event {
                TransactionalOperationStep::CleanTable { table_name } => {
                    let db_table = tables.get(table_name.as_str()).unwrap();
                    crate::db_operations::write::clean_table::execute(
                        app,
                        db_table.clone(),
                        event_src.clone(),
                        persist_moment,
                    )
                    .await;
                }
                TransactionalOperationStep::DeletePartitions {
                    table_name,
                    partition_keys,
                } => {
                    let db_table = tables.get(table_name.as_str()).unwrap();
                    crate::db_operations::write::delete_partitions(
                        app,
                        db_table.as_ref(),
                        partition_keys,
                        event_src.clone(),
                        persist_moment,
                    )
                    .await;
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
                        now.date_time,
                        persist_moment,
                    )
                    .await;
                }
                TransactionalOperationStep::UpdateRows(state) => {
                    let db_table = tables.get(state.table_name.as_str()).unwrap();
                    crate::db_operations::write::bulk_insert_or_update::execute(
                        app,
                        db_table.clone(),
                        state.rows_by_partition,
                        event_src.clone(),
                        now,
                        persist_moment,
                    )
                    .await;
                }
            }
        }
    }

    Ok(())
}
