use std::sync::Arc;

use my_http_server::{HttpOkResult, WebContentType};

use crate::{db::DbRow, db_operations::read::ReadOperationResult, json::JsonArrayBuilder};

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
