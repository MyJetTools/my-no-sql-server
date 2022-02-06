use std::sync::Arc;

use my_http_server::{HttpFailResult, WebContentType};
use my_http_server_controllers::controllers::documentation::{
    data_types::HttpDataType, out_results::HttpResult,
};

use crate::{app::AppContext, data_readers::DataReader};

#[async_trait::async_trait]
pub trait HttpSessionsSupport {
    async fn get_http_session(&self, session_id: &str) -> Result<Arc<DataReader>, HttpFailResult>;
}

#[async_trait::async_trait]
impl HttpSessionsSupport for AppContext {
    async fn get_http_session(&self, session_id: &str) -> Result<Arc<DataReader>, HttpFailResult> {
        if let Some(result) = self.data_readers.get_http(session_id).await {
            return Ok(result);
        }

        let err = HttpFailResult {
            content_type: WebContentType::Text,
            status_code: SESSION_NOT_FOUND_HTTP_CODE,
            content: "Session not found".to_string().into_bytes(),
            write_telemetry: false,
        };

        Err(err)
    }
}

const SESSION_NOT_FOUND_HTTP_CODE: u16 = 403;
pub fn session_not_found_result_description() -> HttpResult {
    HttpResult {
        http_code: SESSION_NOT_FOUND_HTTP_CODE,
        nullable: true,
        description: "Session not found".to_string(),
        data_type: HttpDataType::as_string(),
    }
}
