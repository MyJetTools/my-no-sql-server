use crate::{
    db_operations::DbOperationError,
    http::{http_fail::HttpFailResult, web_content_type::WebContentType},
};

impl From<DbOperationError> for HttpFailResult {
    fn from(src: DbOperationError) -> Self {
        match src {
            DbOperationError::TableAlreadyExists => HttpFailResult {
                content_type: WebContentType::Text,
                status_code: 401,
                content: format!("Table already exists").into_bytes(),
            },
            DbOperationError::TableNotFound(table_name) => HttpFailResult {
                content_type: WebContentType::Text,
                status_code: 401,
                content: format!("Table '{}' not found", table_name).into_bytes(),
            },
            DbOperationError::RecordNotFound => HttpFailResult {
                content_type: WebContentType::Text,
                status_code: 404,
                content: "Record not found".as_bytes().to_vec(),
            },
            DbOperationError::OptimisticConcurencyUpdateFails => HttpFailResult {
                content_type: WebContentType::Text,
                status_code: 403, //TODO - Check the code with the reader
                content: "Record is changed found".as_bytes().to_vec(),
            },
            DbOperationError::RecordAlreadyExists => HttpFailResult {
                content_type: WebContentType::Text,
                status_code: 401,
                content: format!("Record already exists").into_bytes(),
            },
            DbOperationError::TimeStampFieldRequires => HttpFailResult {
                content_type: WebContentType::Text,
                status_code: 401,
                content: format!("Timestamp field requires").into_bytes(),
            },
        }
    }
}
