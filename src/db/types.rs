use std::sync::Arc;

use super::DbRow;

pub enum OperationResult {
    Ok,
    OkWithJsonString { json: String },
    Rows { rows: Option<Vec<Arc<DbRow>>> },
    Row { row: Arc<DbRow> },
    Text { text: String },
    Number { value: i64 },
}

#[derive(Debug)]
pub enum FailOperationResult {
    TableAlreadyExist { table_name: String },
    TableNotFound { table_name: String },
    InvalidJson { err: String },
    FieldPartitionKeyIsRequired,
    FieldRowKeyIsRequired,
    TimeStampFieldRequires,
    //RowKeyNotFound,
    //RecordAlreadyExists,
    RecordNotFound,
    OptimisticConcurencyUpdateFails,
    QueryParameterRequires { param_name: String },
    //AccessViolation { process: String },
}
