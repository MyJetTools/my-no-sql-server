use my_http_server::{HttpOkResult, WebContentType};

use crate::db_operations::write::WriteOperationResult;

impl Into<HttpOkResult> for WriteOperationResult {
    fn into(self) -> HttpOkResult {
        match self {
            WriteOperationResult::SingleRow(db_row) => HttpOkResult::Content {
                content_type: Some(WebContentType::Json),
                content: db_row.data.to_vec(),
            },
            WriteOperationResult::Empty => HttpOkResult::Empty,
        }
    }
}
