use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult};
use my_http_server_controllers::controllers::{
    actions::PostAction,
    documentation::{out_results::HttpResult, HttpActionDescription},
};

use crate::{app::AppContext, http::controllers::row_controller::models::BaseDbRowContract};

use super::models::GetSinglePartitionMultipleRowsActionInputContract;

pub struct GetSinglePartitionMultipleRowsAction {
    app: Arc<AppContext>,
}

impl GetSinglePartitionMultipleRowsAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl PostAction for GetSinglePartitionMultipleRowsAction {
    fn get_route(&self) -> &str {
        "/Rows/SinglePartitionMultipleRows"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: super::consts::CONTROLLER_NAME,
            description: "Get Rows Count",

            input_params: GetSinglePartitionMultipleRowsActionInputContract::get_input_params()
                .into(),
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

    async fn handle_request(&self, ctx: &mut HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let input_data =
            GetSinglePartitionMultipleRowsActionInputContract::parse_http_input(ctx).await?;

        let db_table = crate::db_operations::read::table::get(
            self.app.as_ref(),
            input_data.table_name.as_ref(),
        )
        .await?;

        let row_keys = serde_json::from_slice(input_data.body.as_slice()).unwrap();

        let result = crate::db_operations::read::rows::get_single_partition_multiple_rows(
            db_table.as_ref(),
            input_data.partition_key.as_ref(),
            row_keys,
        )
        .await;

        Ok(result.into())
    }
}
