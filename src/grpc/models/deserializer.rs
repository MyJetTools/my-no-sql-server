use std::{collections::BTreeMap, sync::Arc};

use crate::{
    db_json_entity::JsonTimeStamp,
    db_transactions::steps::{TransactionalOperationStep, UpdateRowsStepState},
    mynosqlserver_grpc::*,
};

use super::{
    CleanTableTransactionActionGrpcModel, DeletePartitionsTransactionActionGrpcModel,
    DeleteRowsTransactionActionGrpcModel, GrpcContractConvertError,
    InsertOrReplaceEntitiesTransactionActionGrpcModel,
};

pub fn deserialize(
    transaction_type: TransactionType,
    content: &[u8],
) -> Result<TransactionalOperationStep, GrpcContractConvertError> {
    match transaction_type {
        TransactionType::CleanTable => {
            let contract = CleanTableTransactionActionGrpcModel::deserialize(content)?;

            let result = TransactionalOperationStep::CleanTable {
                table_name: contract.table_name,
            };
            Ok(result)
        }
        TransactionType::DeletePartitions => {
            let contract = DeletePartitionsTransactionActionGrpcModel::deserialize(content)?;
            let result = TransactionalOperationStep::DeletePartitions {
                table_name: contract.table_name,
                partition_keys: contract.partition_keys,
            };
            Ok(result)
        }

        TransactionType::DeleteRows => {
            let contract = DeleteRowsTransactionActionGrpcModel::deserialize(content)?;
            let result = TransactionalOperationStep::DeleteRows {
                table_name: contract.table_name,
                partition_key: contract.partition_key,
                row_keys: contract.row_keys,
            };
            Ok(result)
        }

        TransactionType::InsertOrReplaceEntities => {
            let contract = InsertOrReplaceEntitiesTransactionActionGrpcModel::deserialize(content)?;

            let time_stamp = JsonTimeStamp::now();

            let mut update_rows_state = UpdateRowsStepState {
                table_name: contract.table_name,
                rows_by_partition: BTreeMap::new(),
            };

            for entity in contract.entities {
                let db_row = entity.to_db_row(&time_stamp)?;

                if update_rows_state
                    .rows_by_partition
                    .contains_key(&db_row.partition_key)
                {
                    update_rows_state
                        .rows_by_partition
                        .insert(db_row.partition_key.to_string(), Vec::new());

                    update_rows_state
                        .rows_by_partition
                        .get_mut(&db_row.partition_key)
                        .unwrap()
                        .push(Arc::new(db_row));
                }
            }

            let result = TransactionalOperationStep::UpdateRows(update_rows_state);

            Ok(result)
        }
    }
}
