use my_no_sql_sdk::core::my_json::json_reader::array_iterator::JsonArrayIterator;
use my_no_sql_server_core::rust_extensions::array_of_bytes_iterator::SliceIterator;

use crate::{
    db_operations::transactions::TransactionOperationError,
    db_transactions::steps::TransactionalOperationStep,
    http::controllers::transactions::models::JsonBaseTransaction,
};

use super::models::{
    CleanTableTransactionJsonModel, DeletePartitionsTransactionJsonModel,
    DeleteRowsTransactionJsonModel, InsertOrUpdateTransactionJsonModel,
};

const JSON_TRANSACTION_CLEAN_TABLE: &str = "CleanTable";
const JSON_TRANSACTION_DELETE_PARTITIONS: &str = "CleanPartitions";
const JSON_TRANSACTION_DELETE_ROWS: &str = "DeleteRows";
const JSON_TRANSACTION_INSERT_OR_UPDATE: &str = "InsertOrUpdate";

pub fn parse_transactions(
    payload: &[u8],
) -> Result<Vec<TransactionalOperationStep>, TransactionOperationError> {
    let mut json_array_iterator: JsonArrayIterator<SliceIterator> = payload.into();

    let mut result = Vec::new();

    while let Some(json_object) = json_array_iterator.get_next() {
        let json_object = json_object.unwrap();
        let type_model: JsonBaseTransaction =
            serde_json::from_slice(json_object.as_bytes().unwrap())?;

        if type_model.transaction_type == JSON_TRANSACTION_CLEAN_TABLE {
            let model: CleanTableTransactionJsonModel =
                serde_json::from_slice(json_object.as_bytes().unwrap())?;

            result.push(model.into())
        }

        if type_model.transaction_type == JSON_TRANSACTION_DELETE_PARTITIONS {
            let model: DeletePartitionsTransactionJsonModel =
                serde_json::from_slice(json_object.as_bytes().unwrap())?;

            result.push(model.into())
        }

        if type_model.transaction_type == JSON_TRANSACTION_DELETE_ROWS {
            let model: DeleteRowsTransactionJsonModel =
                serde_json::from_slice(json_object.as_bytes().unwrap())?;

            result.push(model.into())
        }

        if type_model.transaction_type == JSON_TRANSACTION_INSERT_OR_UPDATE {
            let model: InsertOrUpdateTransactionJsonModel =
                serde_json::from_slice(json_object.as_bytes().unwrap())?;

            result.push(model.into()?)
        }
    }

    Ok(result)
}
