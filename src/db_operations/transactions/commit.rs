use std::collections::HashMap;

use crate::{
    app::AppContext, db_sync::SyncAttributes, db_transactions::steps::TransactionalOperationStep,
};

use super::TransactionOperationError;

pub async fn commit(
    app: &AppContext,
    transaction_id: &str,
    attr: SyncAttributes,
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
                        Some(attr.clone()),
                    )
                    .await;
                }
                TransactionalOperationStep::DeletePartitions {
                    table_name,
                    partition_keys,
                } => {
                    let db_table = tables.get(table_name.as_str()).unwrap();
                    crate::db_operations::write::delete_partitions::execute(
                        app,
                        db_table.clone(),
                        partition_keys,
                        Some(attr.clone()),
                    )
                    .await;
                }
                TransactionalOperationStep::DeleteRows {
                    table_name,
                    partition_key,
                    row_keys,
                } => {
                    let db_table = tables.get(table_name.as_str()).unwrap();
                    crate::db_operations::write::delete_rows::execute(
                        app,
                        db_table.clone(),
                        partition_key.as_str(),
                        row_keys,
                        Some(attr.clone()),
                    )
                    .await;
                }
                TransactionalOperationStep::UpdateRows(state) => {
                    let db_table = tables.get(state.table_name.as_str()).unwrap();
                    crate::db_operations::write::bulk_insert_or_update::execute(
                        app,
                        db_table.clone(),
                        state.rows_by_partition,
                        Some(attr.clone()),
                    )
                    .await;
                }
            }
        }
    }

    Ok(())
}
