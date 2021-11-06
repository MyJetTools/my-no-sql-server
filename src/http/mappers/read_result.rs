use my_http_utils::WebContentType;

use crate::{db_operations::read::ReadOperationResult, http::http_ok::HttpOkResult};

impl Into<HttpOkResult> for ReadOperationResult {
    fn into(self) -> HttpOkResult {
        match self {
            ReadOperationResult::SingleRow(content) => HttpOkResult::Content {
                content,
                content_type: Some(WebContentType::Json),
            },
            ReadOperationResult::RowsArray(content) => HttpOkResult::Content {
                content,
                content_type: Some(WebContentType::Json),
            },
            ReadOperationResult::EmptyArray => {
                let empty_array = vec![
                    crate::json::consts::OPEN_ARRAY,
                    crate::json::consts::CLOSE_ARRAY,
                ];

                HttpOkResult::Content {
                    content: empty_array,
                    content_type: Some(WebContentType::Json),
                }
            }
        }
    }
}
