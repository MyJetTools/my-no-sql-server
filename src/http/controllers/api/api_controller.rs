use async_trait::async_trait;
use rust_extensions::date_time::DateTimeAsMicroseconds;

use my_http_server::{
    middlewares::controllers::{
        actions::GetAction,
        documentation::{
            data_types::{HttpDataType, HttpField, HttpObjectStructure},
            out_results::HttpResult,
            HttpActionDescription,
        },
    },
    HttpContext, HttpFailResult, HttpOkResult,
};

use super::models::ApiModel;

pub struct ApiController {}

impl ApiController {
    pub fn new() -> Self {
        Self {}
    }
}
#[async_trait]
impl GetAction for ApiController {
    fn get_additional_types(&self) -> Option<Vec<HttpObjectStructure>> {
        None
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: "Monitoring",
            description: "Monitoring API",

            input_params: None,
            results: vec![HttpResult {
                http_code: 200,
                nullable: true,
                description: "Get Monitoring structure".to_string(),
                data_type: HttpDataType::Object(HttpObjectStructure {
                    struct_id: "IsAliveResponse".to_string(),
                    fields: vec![
                        HttpField::new("name", HttpDataType::as_string(), true, None),
                        HttpField::new("time", HttpDataType::as_date_time(), true, None),
                        HttpField::new("version", HttpDataType::as_string(), true, None),
                        HttpField::new("env_info", HttpDataType::as_string(), false, None),
                    ],
                }),
            }],
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

        let model = ApiModel {
            name: "MyNoSqlServer".to_string(),
            time: time.to_rfc3339(),
            version: version.to_string(),
            env_info,
        };

        return HttpOkResult::create_json_response(model).into();
    }
}
