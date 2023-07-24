use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};

use super::models::GetPartitionsAmountContract;
use crate::app::AppContext;
use std::{result::Result, sync::Arc};

#[my_http_server_swagger::http_route(
    method: "GET",
    route: "/Partitions/Count",
    input_data: "GetPartitionsAmountContract",
    description: "Get Partitions amount of selected table",
    summary: "Returns Partitions amount of selected table",
    controller: "Partitions",
    result:[
        {status_code: 200, description: "Partitions amount", model: "Long"},
        {status_code: 400, description: "Table not found"},
    ]
)]
pub struct GetPartitionsCountAction {
    app: Arc<AppContext>,
}

impl GetPartitionsCountAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &GetPartitionsCountAction,
    input_data: GetPartitionsAmountContract,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let db_table =
        crate::db_operations::read::table::get(action.app.as_ref(), input_data.table_name.as_str())
            .await?;

    let partitions_amount = db_table.get_partitions_amount().await;

    HttpOutput::as_text(format!("{}", partitions_amount))
        .into_ok_result(true)
        .into()
}
