use my_http_server::{HttpFailResult, HttpOkResult, HttpOutput, WebContentType};

use crate::db_operations::write::WriteOperationResult;

impl Into<Result<HttpOkResult, HttpFailResult>> for WriteOperationResult {
    fn into(self) -> Result<HttpOkResult, HttpFailResult> {
        match self {
            WriteOperationResult::SingleRow(db_row) => {
                let mut content = Vec::new();
                db_row.compile_json(&mut content);
                let output = HttpOutput::Content {
                    headers: None,
                    content_type: Some(WebContentType::Json),
                    content,
                };

                Ok(HttpOkResult {
                    write_telemetry: false,
                    output,
                })
            }
            WriteOperationResult::Empty => HttpOutput::Empty.into_ok_result(true),
        }
    }
}
