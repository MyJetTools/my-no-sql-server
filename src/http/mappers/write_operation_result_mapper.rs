use my_http_server::{HttpOkResult, HttpOutput, WebContentType};

use crate::db_operations::write::WriteOperationResult;

impl Into<HttpOkResult> for WriteOperationResult {
    fn into(self) -> HttpOkResult {
        match self {
            WriteOperationResult::SingleRow(db_row) => {
                let output = HttpOutput::Content {
                    headers: None,
                    content_type: Some(WebContentType::Json),
                    content: db_row.data.to_vec(),
                };

                HttpOkResult {
                    write_telemetry: false,
                    output,
                }
            }
            WriteOperationResult::Empty => HttpOutput::Empty.into_ok_result(true),
        }
    }
}
