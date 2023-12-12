use my_http_server::macros::*;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};
use std::sync::Arc;

use crate::{app::AppContext, http::http_sessions::HttpSessionsSupport};

use super::models::PingInputModel;

#[http_route(
    method: "POST",
    route: "/api/DataReader/Ping",
    deprecated_routes: ["/DataReader/Ping"],
    controller: "DataReader",
    summary: "Pings that subscriber is alive",
    description: "Pings that subscriber is alive",
    input_data: "PingInputModel",
    result:[
        {status_code: 202, description: "Successful operation"},
    ]
)]
pub struct PingAction {
    app: Arc<AppContext>,
}

impl PingAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

/*
#[async_trait::async_trait]
impl PostAction for PingAction {
    fn get_route(&self) -> &str {
        "/DataReader/Ping"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: super::consts::CONTROLLER_NAME,
            description: "Get Subscriber changes",

            input_params: PingInputModel::get_input_params().into(),
            results: vec![HttpResult {
                http_code: 202,
                nullable: true,
                description: "Successful operation".to_string(),
                data_type: HttpDataType::None,
            }],
        }
        .into()
    }

    async fn handle_request(&self, ctx: &mut HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let input_data = PingInputModel::parse_http_input(ctx).await?;

        self.app
            .get_http_session(input_data.session_id.as_str())
            .await?;

        HttpOutput::Empty.into_ok_result(true).into()
    }
}
 */

async fn handle_request(
    action: &PingAction,
    input_data: PingInputModel,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    action
        .app
        .get_http_session(input_data.session_id.as_str())
        .await?;

    HttpOutput::Empty.into_ok_result(true).into()
}
