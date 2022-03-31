use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult};
use my_http_server_controllers::controllers::actions::GetAction;
use my_http_server_controllers::controllers::documentation::out_results::HttpResult;
use my_http_server_controllers::controllers::documentation::HttpActionDescription;

use crate::app::AppContext;

use super::models::{BaseDbRowContract, GetRowAndUpdateExpirationTimeInputModel};

pub struct RowAndUpdateExpirationTimeAction {
    app: Arc<AppContext>,
}

impl RowAndUpdateExpirationTimeAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl GetAction for RowAndUpdateExpirationTimeAction {
    fn get_route(&self) -> &str {
        "/Row/WithUpdateExpirationTime"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: super::consts::CONTROLLER_NAME,
            description: "Get Entities",

            input_params: GetRowAndUpdateExpirationTimeInputModel::get_input_params().into(),
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
        let http_input = GetRowAndUpdateExpirationTimeInputModel::parse_http_input(ctx).await?;

        let db_table = crate::db_operations::read::table::get(
            self.app.as_ref(),
            http_input.table_name.as_ref(),
        )
        .await?;

        let expiration_time = if let Some(expiration_time) = &http_input.expiration_time {
            crate::json::date_time::parse(expiration_time.as_bytes())
        } else {
            None
        };

        if let Some(partition_key) = http_input.partition_key.as_ref() {
            if let Some(row_key) = http_input.row_key.as_ref() {
                let result =
                    crate::db_operations::read::rows::get_single_and_update_expiration_time(
                        db_table.as_ref(),
                        partition_key,
                        row_key,
                        expiration_time,
                    )
                    .await?;

                return Ok(result.into());
            } else {
                let result = crate::db_operations::read::rows::get_all_by_partition_key_and_update_expiration_time(
                    db_table.as_ref(),
                    partition_key,
                    http_input.limit,
                    http_input.skip,
                    expiration_time
                )
                .await;

                return Ok(result.into());
            }
        } else {
            if let Some(row_key) = http_input.row_key.as_ref() {
                let result = crate::db_operations::read::rows::get_all_by_row_key_and_update_expiration_time(
                    db_table.as_ref(),
                    row_key,
                    http_input.limit,
                    http_input.skip,
                    expiration_time,
                )
                .await;

                return Ok(result.into());
            } else {
                let result = crate::db_operations::read::rows::get_all_and_update_expiration_time(
                    db_table.as_ref(),
                    http_input.limit,
                    http_input.skip,
                    expiration_time,
                )
                .await;

                return Ok(result.into());
            }
        }
    }
}
