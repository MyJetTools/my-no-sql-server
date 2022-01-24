use async_trait::async_trait;
use rust_extensions::date_time::DateTimeAsMicroseconds;

use my_http_server::{
    middlewares::controllers::{
        actions::GetAction,
        documentation::{out_results::IntoHttpResult, HttpActionDescription},
    },
    HttpContext, HttpFailResult, HttpOkResult,
};

use super::models::IsAliveResponse;

pub struct ApiController {}

impl ApiController {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl GetAction for ApiController {
    fn get_route(&self) -> &str {
        "/Api/IsAlive"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: "Monitoring",
            description: "Monitoring API",

            input_params: None,
            results: vec![
                IsAliveResponse::get_http_data_structure().into_http_result_object(
                    200,
                    false,
                    "Monitoring result",
                ),
            ],
        }
        .into()
    }

    async fn handle_request(&self, ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let version = env!("CARGO_PKG_VERSION");

        let env_info = match std::env::var("ENV_INFO") {
            Ok(value) => Some(value),
            Err(_) => None,
        };

        let time = DateTimeAsMicroseconds::now();

        let model = IsAliveResponse {
            name: "MyNoSqlServer".to_string(),
            time: time.to_rfc3339(),
            version: version.to_string(),
            env_info,
        };

        return HttpOkResult::create_json_response(model).into();
    }
}
