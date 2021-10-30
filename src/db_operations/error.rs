pub enum DbOperationError {
    TableNotFound(String),
    TableAlreadyExists,
    RecordAlreadyExists,
    TimeStampFieldRequires,
    RecordNotFound,
    OptimisticConcurencyUpdateFails,
}
