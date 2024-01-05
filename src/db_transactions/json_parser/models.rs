use std::{collections::BTreeMap, sync::Arc};

use my_no_sql_sdk::core::db_json_entity::{DbEntityParseFail, DbJsonEntity, JsonTimeStamp};
use serde::{Deserialize, Serialize};

use crate::db_transactions::steps::{TransactionalOperationStep, UpdateRowsStepState};

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

#[derive(Serialize, Deserialize, Debug)]
pub struct InsertOrUpdateTransactionJsonModel {
    #[serde(rename = "tableName")]
    pub table_name: String,

    #[serde(rename = "entities")]
    pub entities: Vec<Vec<u8>>,
}

impl InsertOrUpdateTransactionJsonModel {
    pub fn into(self) -> Result<TransactionalOperationStep, DbEntityParseFail> {
        let mut rows_by_partition = BTreeMap::new();

        let now = JsonTimeStamp::now();

        for entity in self.entities {
            let db_entity = DbJsonEntity::parse(&entity)?;
            let db_row = Arc::new(db_entity.into_db_row(entity, &now));

            let partition_key = db_row.get_partition_key();

            if !rows_by_partition.contains_key(partition_key) {
                rows_by_partition.insert(partition_key.to_string(), Vec::new());
            }

            rows_by_partition
                .get_mut(partition_key)
                .unwrap()
                .push(db_row);
        }

        let state = UpdateRowsStepState {
            table_name: self.table_name,
            rows_by_partition,
        };

        let result = TransactionalOperationStep::UpdateRows(state);

        Ok(result)
    }
}
