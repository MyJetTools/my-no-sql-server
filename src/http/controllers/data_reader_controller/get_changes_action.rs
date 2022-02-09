use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput, WebContentType};
use my_http_server_controllers::controllers::{
    actions::PostAction,
    documentation::{data_types::HttpDataType, out_results::HttpResult, HttpActionDescription},
};

use crate::{
    app::AppContext,
    data_readers::{http_connection::HttpPayload, DataReaderConnection},
    http::http_sessions::HttpSessionsSupport,
};

use super::models::GetChangesInputModel;

pub struct GetChangesAction {
    app: Arc<AppContext>,
}

impl GetChangesAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}
#[async_trait::async_trait]
impl PostAction for GetChangesAction {
    fn get_route(&self) -> &str {
        "/DataReader/GetChanges"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: super::consts::CONTROLLER_NAME,
            description: "Get Subscriber changes",

            input_params: GetChangesInputModel::get_input_params().into(),
            results: vec![HttpResult {
                http_code: 200,
                nullable: true,
                description: "Successful operation".to_string(),
                data_type: HttpDataType::None,
            }],
        }
        .into()
    }

    async fn handle_request(&self, ctx: &mut HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let input_data = GetChangesInputModel::parse_http_input(ctx).await?;

        let data_reader = self
            .app
            .get_http_session(input_data.session_id.as_str())
            .await?;

        if let DataReaderConnection::Http(info) = &data_reader.connection {
            let result = info.new_request().await?;
            match result {
                HttpPayload::Ping => return HttpOutput::Empty.into_ok_result(false).into(),
                HttpPayload::Payload(payload) => {
                    return HttpOutput::Content {
                        headers: None,
                        content_type: None,
                        content: payload,
                    }
                    .into_ok_result(false)
                    .into();
                }
            }
        }

        return Err(HttpFailResult {
            content_type: WebContentType::Text,
            status_code: 400,
            content: "Only HTTP sessions are supported".to_string().into_bytes(),
            write_telemetry: true,
        });
    }
}
