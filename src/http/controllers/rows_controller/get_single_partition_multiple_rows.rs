use my_http_server::macros::*;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult};
use std::sync::Arc;

use crate::{app::AppContext, http::controllers::row_controller::models::BaseDbRowContract};

use super::models::GetSinglePartitionMultipleRowsActionInputContract;

#[http_route(
    method: "POST",
    route: "/api/Rows/SinglePartitionMultipleRows",
    deprecated_routes: ["/Rows/SinglePartitionMultipleRows"],
    controller: "Rows",
    description: "Return specific rows from the partition",
    summary: "Returns specific rows from the partition",
    input_data: "GetSinglePartitionMultipleRowsActionInputContract",
    result:[
        {status_code: 200, description: "Rows", model: "Vec<BaseDbRowContract>"},
    ]
)]
pub struct GetSinglePartitionMultipleRowsAction {
    app: Arc<AppContext>,
}

impl GetSinglePartitionMultipleRowsAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &GetSinglePartitionMultipleRowsAction,
    input_data: GetSinglePartitionMultipleRowsActionInputContract,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let db_table =
        crate::db_operations::read::table::get(action.app.as_ref(), input_data.table_name.as_ref())
            .await?;

    let row_keys = serde_json::from_slice(input_data.body.as_slice()).unwrap();

    let result = crate::db_operations::read::rows::get_single_partition_multiple_rows(
        &action.app,
        &db_table,
        &input_data.partition_key,
        row_keys,
        input_data.get_update_statistics(),
    )
    .await?;

    Ok(result.into())
}
