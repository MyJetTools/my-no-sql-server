use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult};

use crate::{app::AppContext, http::controllers::row_controller::models::BaseDbRowContract};

use super::models::GetHighestRowsAndBelowInputContract;

#[my_http_server_swagger::http_route(
    method: "GET",
    route: "/Rows/HighestRowAndBelow",
    controller: "Rows",
    description: "Return rows from highes db_row and below",
    summary: "Return rows from highes db_row and below",
    input_data: "GetHighestRowsAndBelowInputContract",
    result:[
        {status_code: 200, description: "Rows", model: "Vec<BaseDbRowContract>"},
    ]
)]
pub struct GetHighestRowAndBelowAction {
    app: Arc<AppContext>,
}

impl GetHighestRowAndBelowAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

/*
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
}

 */

async fn handle_request(
    action: &GetHighestRowAndBelowAction,
    input_data: GetHighestRowsAndBelowInputContract,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let db_table =
        crate::db_operations::read::table::get(action.app.as_ref(), input_data.table_name.as_ref())
            .await?;

    let limit = if let Some(max_amount) = input_data.max_amount {
        if max_amount == 0 {
            None
        } else {
            Some(max_amount)
        }
    } else {
        None
    };

    let result = crate::db_operations::read::get_highest_row_and_below(
        &action.app,
        &db_table,
        &input_data.partition_key,
        &input_data.row_key,
        limit,
        input_data.get_update_statistics(),
    )
    .await?;

    Ok(result.into())
}
