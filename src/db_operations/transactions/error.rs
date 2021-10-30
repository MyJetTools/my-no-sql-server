use crate::{db_json_entity::DbEntityParseFail, db_operations::DbOperationError};

pub enum TransactionOperationError {
    TransactionNotFound { id: String },
    DbEntityParseFail(DbEntityParseFail),
    DbOperationError(DbOperationError),
    SerdeJsonError(serde_json::Error),
}

impl From<DbEntityParseFail> for TransactionOperationError {
    fn from(src: DbEntityParseFail) -> Self {
        Self::DbEntityParseFail(src)
    }
}

impl From<DbOperationError> for TransactionOperationError {
    fn from(src: DbOperationError) -> Self {
        Self::DbOperationError(src)
    }
}

impl From<serde_json::Error> for TransactionOperationError {
    fn from(src: serde_json::Error) -> Self {
        Self::SerdeJsonError(src)
    }
}
