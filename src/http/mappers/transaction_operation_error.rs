use my_http_server::{HttpFailResult, WebContentType};

use crate::db_operations::transactions::TransactionOperationError;

impl From<TransactionOperationError> for HttpFailResult {
    fn from(src: TransactionOperationError) -> Self {
        match src {
            TransactionOperationError::TransactionNotFound { id } => HttpFailResult {
                content: format!("Transaction {} not found", id).into_bytes(),
                content_type: WebContentType::Text,
                status_code: 401,
                write_telemetry: true,
            },
            TransactionOperationError::DbEntityParseFail(err) => err.into(),
            TransactionOperationError::DbOperationError(op_err) => op_err.into(),
            TransactionOperationError::SerdeJsonError(err) => HttpFailResult {
                content: format!("{}", err).into_bytes(),
                content_type: WebContentType::Text,
                status_code: 500,
                write_telemetry: true,
            },
        }
    }
}
