use std::sync::Arc;

use my_http_server::{
    middlewares::controllers::{
        actions::GetAction,
        documentation::{out_results::HttpResult, HttpActionDescription},
    },
    HttpContext, HttpFailResult, HttpOkResult,
};

use crate::{app::AppContext, http::controllers::row_controller::models::BaseDbRowContract};

use super::models::GetHighestRowsAndBelowInputContract;

pub struct GetHighestRowAndBelowAction {
    app: Arc<AppContext>,
}

impl GetHighestRowAndBelowAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl GetAction for GetHighestRowAndBelowAction {
    fn get_route(&self) -> &str {
        "/Rows/HighestRowAndBelow"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: super::consts::CONTROLLER_NAME,
            description: "Get Rows Count",

            input_params: GetHighestRowsAndBelowInputContract::get_input_params().into(),
            results: vec![
                HttpResult {
                    http_code: 200,
                    nullable: false,
                    description: "Rows".to_string(),
                    data_type: BaseDbRowContract::get_http_data_structure()
                        .into_http_data_type_array(),
                },
                crate::http::docs::rejects::op_with_table_is_failed(),
            ],
        }
        .into()
    }

    async fn handle_request(&self, ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let input_data = GetHighestRowsAndBelowInputContract::parse_http_input(ctx).await?;

        let db_table = crate::db_operations::read::table::get(
            self.app.as_ref(),
            input_data.table_name.as_ref(),
        )
        .await?;

        let result = crate::db_operations::read::get_highest_row_and_below::execute(
            db_table.as_ref(),
            input_data.partition_key.as_ref(),
            &input_data.row_key,
            input_data.max_amount,
        )
        .await;

        Ok(result.into())
    }
}
