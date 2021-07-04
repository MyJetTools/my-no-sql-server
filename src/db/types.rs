use std::sync::Arc;

use serde::Serialize;

use crate::json::JsonParseError;

use super::DbRow;

pub enum OperationResult {
    Ok,
    Json { json: String },
    Rows { rows: Option<Vec<Arc<DbRow>>> },
    Row { row: Arc<DbRow> },
    Text { text: String },
    Html { title: String, body: String },
    Number { value: i64 },
}

impl OperationResult {
    pub fn create_json_response<T: Serialize>(
        model: T,
    ) -> Result<OperationResult, FailOperationResult> {
        let json = serde_json::to_string(&model).unwrap();
        Ok(OperationResult::Json { json })
    }
}

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
