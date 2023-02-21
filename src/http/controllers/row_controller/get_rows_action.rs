use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult};

use super::models::*;
use crate::app::AppContext;

#[my_http_server_swagger::http_route(
    method: "GET",
    route: "/Row",
    controller: "Row",
    description: "Get Entitity or entities",
    summary: "Returns Entitity or entities",
    input_data: "GetRowInputModel",
    result:[
        {status_code: 200, description: "Single Row or array of rows"},
    ]
)]
pub struct GetRowsAction {
    app: Arc<AppContext>,
}

impl GetRowsAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

/*

#[async_trait::async_trait]
impl GetAction for RowAction {
    fn get_route(&self) -> &str {
        "/Row"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: super::consts::CONTROLLER_NAME,
            description: "Get Entities",

            input_params: GetRowInputModel::get_input_params().into(),
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
}

}
 */

async fn handle_request(
    action: &GetRowsAction,
    input_data: GetRowInputModel,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let db_table =
        crate::db_operations::read::table::get(action.app.as_ref(), input_data.table_name.as_ref())
            .await?;

    if let Some(partition_key) = input_data.partition_key.as_ref() {
        if let Some(row_key) = input_data.row_key.as_ref() {
            let result = crate::db_operations::read::rows::get_single(
                action.app.as_ref(),
                &db_table,
                partition_key,
                row_key,
                input_data.get_update_statistics(),
            )
            .await?;

            return Ok(result.into());
        } else {
            let result = crate::db_operations::read::rows::get_all_by_partition_key(
                &action.app,
                &db_table,
                partition_key,
                input_data.limit,
                input_data.skip,
                input_data.get_update_statistics(),
            )
            .await?;

            return Ok(result.into());
        }
    } else {
        if let Some(row_key) = input_data.row_key.as_ref() {
            let result = crate::db_operations::read::rows::get_all_by_row_key(
                &action.app,
                &db_table,
                row_key,
                input_data.limit,
                input_data.skip,
                input_data.get_update_statistics(),
            )
            .await?;

            return Ok(result.into());
        } else {
            let result = crate::db_operations::read::rows::get_all(
                &action.app,
                &db_table,
                input_data.limit,
                input_data.skip,
                input_data.get_update_statistics(),
            )
            .await?;

            return Ok(result.into());
        }
    }
}
