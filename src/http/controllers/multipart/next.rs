use std::sync::Arc;

use my_http_server::{
    middlewares::controllers::{
        actions::PostAction,
        documentation::{out_results::IntoHttpResult, HttpActionDescription},
    },
    HttpContext, HttpFailResult, HttpOkResult,
};

use crate::app::AppContext;

use super::models::{BaseDbRow, NextMultipartRequestInputContract};

pub struct NextMultipartController {
    app: Arc<AppContext>,
}

impl NextMultipartController {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl PostAction for NextMultipartController {
    fn get_route(&self) -> &str {
        "/Multipart/Next"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: super::consts::CONTROLLER_NAME,
            description: "Monitoring API",

            input_params: Some(NextMultipartRequestInputContract::get_input_params()),
            results: vec![BaseDbRow::get_http_data_structure().into_http_result_array(
                200,
                false,
                "New multipart is started",
            )],
        }
        .into()
    }
    async fn handle_request(&self, ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let input_data = NextMultipartRequestInputContract::parse_http_input(ctx).await?;

        let db_rows = crate::db_operations::read::multipart::get_next(
            self.app.as_ref(),
            input_data.request_id,
            input_data.max_records_count,
        )
        .await;

        if db_rows.is_none() {
            return Err(HttpFailResult::as_not_found("".to_string(), false));
        }

        return Ok(db_rows.unwrap().into());
    }
}
