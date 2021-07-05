use std::sync::Arc;

use serde::Serialize;

use crate::json::JsonParseError;

use super::DbRow;

#[derive(Debug)]
pub enum FailOperationResult {
    TableAlreadyExist { table_name: String },
    TableNotFound { table_name: String },
    JsonParseError(crate::json::JsonParseError),
    FieldPartitionKeyIsRequired,
    FieldRowKeyIsRequired,
    TimeStampFieldRequires,
    //RowKeyNotFound,
    //RecordAlreadyExists,
    RecordNotFound,
    OptimisticConcurencyUpdateFails,
    QueryParameterRequires { param_name: String },
    TransactionNotFound { id: String },
    //AccessViolation { process: String },
}

impl From<JsonParseError> for FailOperationResult {
    fn from(value: JsonParseError) -> Self {
        Self::JsonParseError(value)
    }
}
