#[derive(Debug)]
pub enum DbOperationError {
    TableNotFound(String),
    TableAlreadyExists,
    RecordAlreadyExists,
    TimeStampFieldRequires,
    RecordNotFound,
    OptimisticConcurencyUpdateFails,
    TableNameValidationError(String),
    ApplicationIsNotInitializedYet,
}
