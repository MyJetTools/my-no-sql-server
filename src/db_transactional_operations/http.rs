use std::collections::HashMap;

use crate::{
    app::AppServices,
    db::{DbRow, FailOperationResult},
    db_entity::DbEntity,
    db_transactions::TransactionAttributes,
    json::array_parser::ArrayToJsonObjectsSplitter,
};

use serde::{Deserialize, Serialize};

use super::transactional_operation::TransactionalOperationStep;

pub async fn appen_events(
    app: &AppServices,
    transaction_id: &str,
    payload: Vec<u8>,
) -> Result<(), FailOperationResult> {
    let transactions = parse_transactions(payload.as_slice())?;
    app.active_transactions
        .add_events(transaction_id, transactions)
        .await;

    Ok(())
}

pub async fn commit(
    app: &AppServices,
    transaction_id: &str,
    attr: TransactionAttributes,
) -> Result<(), FailOperationResult> {
    let transaction = app.active_transactions.remove(transaction_id).await;

    if transaction.is_none() {
        return Err(FailOperationResult::TransactionNotFound {
            id: transaction_id.to_string(),
        });
    }

    let mut transaction = transaction.unwrap();

    let mut tables = HashMap::new();

    for table_name in transaction.operations.keys() {
        let db_table = app.db.get_table(table_name).await?;
        tables.insert(table_name.to_string(), db_table);
    }

    for (_, mut events) in transaction.operations.drain() {
        for event in events.drain(..) {
            match event {
                TransactionalOperationStep::CleanTable { table_name } => {
                    let db_table = tables.get(table_name.as_str()).unwrap();
                    crate::db_operations::rows::clean_table(
                        app,
                        db_table.as_ref(),
                        Some(attr.clone()),
                    )
                    .await;
                }
                TransactionalOperationStep::DeletePartitions {
                    table_name,
                    partition_keys,
                } => {
                    let db_table = tables.get(table_name.as_str()).unwrap();
                    crate::db_operations::rows::delete_partitions(
                        app,
                        db_table.as_ref(),
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
                    crate::db_operations::rows::delete_rows(
                        app,
                        db_table.as_ref(),
                        partition_key,
                        row_keys,
                        Some(attr.clone()),
                    )
                    .await;
                }
                TransactionalOperationStep::UpdateRows {
                    table_name,
                    rows_by_partition,
                } => {
                    let db_table = tables.get(table_name.as_str()).unwrap();

                    crate::db_operations::rows::bulk_insert_or_update_execute(
                        app,
                        db_table,
                        rows_by_partition,
                        Some(attr.clone()),
                    )
                    .await;
                }
            }
        }
    }

    Ok(())
}

fn parse_transactions(
    payload: &[u8],
) -> Result<Vec<TransactionalOperationStep>, FailOperationResult> {
    let mut result = Vec::new();
    for json_object in payload.split_array_json_to_objects() {
        let type_model: JsonBaseTransaction = serde_json::from_slice(json_object).unwrap();

        if type_model.transaction_type == JSON_TRANSACTION_CLEAN_TABLE {
            let model: CleanTableTransactionJsonModel =
                serde_json::from_slice(json_object).unwrap();

            result.push(model.into())
        }

        if type_model.transaction_type == JSON_TRANSACTION_DELETE_PARTITIONS {
            let model: DeletePartitionsTransactionJsonModel =
                serde_json::from_slice(json_object).unwrap();

            result.push(model.into())
        }

        if type_model.transaction_type == JSON_TRANSACTION_DELETE_ROWS {
            let model: DeleteRowsTransactionJsonModel =
                serde_json::from_slice(json_object).unwrap();

            result.push(model.into())
        }

        if type_model.transaction_type == JSON_TRANSACTION_INSERT_OR_UPDATE {
            let model: InsertOrUpdateTransactionJsonModel =
                serde_json::from_slice(json_object).unwrap();

            result.push(model.into()?)
        }
    }

    Ok(result)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonBaseTransaction {
    #[serde(rename = "type")]
    pub transaction_type: String,
}

const JSON_TRANSACTION_CLEAN_TABLE: &str = "CleanTable";
#[derive(Serialize, Deserialize, Debug)]
pub struct CleanTableTransactionJsonModel {
    #[serde(rename = "tableName")]
    pub table_name: String,
}

impl Into<TransactionalOperationStep> for CleanTableTransactionJsonModel {
    fn into(self) -> TransactionalOperationStep {
        TransactionalOperationStep::CleanTable {
            table_name: self.table_name,
        }
    }
}

///

const JSON_TRANSACTION_DELETE_PARTITIONS: &str = "CleanPartitions";
#[derive(Serialize, Deserialize, Debug)]
pub struct DeletePartitionsTransactionJsonModel {
    #[serde(rename = "tableName")]
    pub table_name: String,
    #[serde(rename = "partitionKeys")]
    pub partition_keys: Vec<String>,
}

impl Into<TransactionalOperationStep> for DeletePartitionsTransactionJsonModel {
    fn into(self) -> TransactionalOperationStep {
        TransactionalOperationStep::DeletePartitions {
            table_name: self.table_name,
            partition_keys: self.partition_keys,
        }
    }
}

/////////

const JSON_TRANSACTION_DELETE_ROWS: &str = "DeleteRows";
#[derive(Serialize, Deserialize, Debug)]
pub struct DeleteRowsTransactionJsonModel {
    #[serde(rename = "tableName")]
    pub table_name: String,
    #[serde(rename = "partitionKey")]
    pub partition_key: String,

    #[serde(rename = "rowKeys")]
    pub row_keys: Vec<String>,
}

impl Into<TransactionalOperationStep> for DeleteRowsTransactionJsonModel {
    fn into(self) -> TransactionalOperationStep {
        TransactionalOperationStep::DeleteRows {
            table_name: self.table_name,
            partition_key: self.partition_key,
            row_keys: self.row_keys,
        }
    }
}

/////////

const JSON_TRANSACTION_INSERT_OR_UPDATE: &str = "InsertOrUpdate";
#[derive(Serialize, Deserialize, Debug)]
pub struct InsertOrUpdateTransactionJsonModel {
    #[serde(rename = "tableName")]
    pub table_name: String,

    #[serde(rename = "entities")]
    pub entities: Vec<Vec<u8>>,
}

impl InsertOrUpdateTransactionJsonModel {
    pub fn into(self) -> Result<TransactionalOperationStep, FailOperationResult> {
        let mut rows_by_partition = HashMap::new();

        for entity in &self.entities {
            let db_entity = DbEntity::parse(entity)?;
            let db_row = DbRow::form_db_entity(&db_entity);

            if !rows_by_partition.contains_key(db_entity.partition_key.as_str()) {
                rows_by_partition.insert(db_entity.partition_key.to_string(), Vec::new());
            }

            rows_by_partition
                .get_mut(db_entity.partition_key.as_str())
                .unwrap()
                .push(db_row);
        }

        let result = TransactionalOperationStep::UpdateRows {
            table_name: self.table_name,
            rows_by_partition,
        };

        Ok(result)
    }
}
