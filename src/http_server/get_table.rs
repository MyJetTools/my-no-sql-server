use my_http_server::{HttpFailResult, HttpOutput, WebContentType};

use crate::http_server::mappers::db_operation_error::OPERATION_FAIL_HTTP_STATUS_CODE;

use super::mappers::{OperationFailHttpContract, OperationFailReason};

pub fn table_not_found_http_result(table_name: &str) -> HttpFailResult {
    let err_model = OperationFailHttpContract {
        reason: OperationFailReason::TableNotFound,
        message: format!("Table '{}' not found", table_name),
    };
    let content = serde_json::to_vec(&err_model).unwrap();

    HttpOutput::Content {
        content_type: Some(WebContentType::Json),
        status_code: OPERATION_FAIL_HTTP_STATUS_CODE,
        content,
        headers: Default::default(),
        set_cookies: Default::default(),
    }
    .into_http_fail_result(true, true)
}
