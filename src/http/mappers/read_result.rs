use std::sync::Arc;

use my_http_server::{HttpOkResult, IntoHttpOkResult, WebContentType};

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

impl IntoHttpOkResult for &DbRow {
    fn into_http_ok_result(&self) -> HttpOkResult {
        HttpOkResult::Content {
            content_type: Some(WebContentType::Json),
            content: self.content.to_vec(),
        }
    }
}

pub fn into_http_ok_result(db_rows: &[Arc<DbRow>]) -> HttpOkResult {
    let mut json_result = JsonArrayBuilder::new();

    for db_row in db_rows {
        json_result.append_json_object(&db_row.data);
    }

    HttpOkResult::Content {
        content_type: Some(WebContentType::Json),
        content: json_result.build(),
    }
}
