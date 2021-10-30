#[derive(Debug)]
pub enum FailOperationResult {
    OptimisticConcurencyUpdateFails,
    QueryParameterRequires { param_name: String },
}
