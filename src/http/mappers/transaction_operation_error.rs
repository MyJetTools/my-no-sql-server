use crate::{
    db_operations::transactions::TransactionOperationError, http::http_fail::HttpFailResult,
};

impl From<TransactionOperationError> for HttpFailResult {
    fn from(src: TransactionOperationError) -> Self {
        match src {
            TransactionOperationError::TransactionNotFound { id } => HttpFailResult {
                content: format!("Transaction {} not found", id).into_bytes(),
                content_type: crate::http::web_content_type::WebContentType::Text,
                status_code: 401,
            },
            TransactionOperationError::DbEntityParseFail(err) => err.into(),
            TransactionOperationError::DbOperationError(op_err) => op_err.into(),
            TransactionOperationError::SerdeJsonError(err) => HttpFailResult {
                content: format!("{}", err).into_bytes(),
                content_type: crate::http::web_content_type::WebContentType::Text,
                status_code: 500,
            },
        }
    }
}
