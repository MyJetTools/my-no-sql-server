use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};

use crate::app::AppContext;

use super::models::{RequestActionInputContract, RequestContract};

#[my_http_server_swagger::http_route(
    method: "GET",
    route: "/Monitoring/Requests",
    input_data: "RequestActionInputContract",
    description: "Get Requests statistic",
    controller: "Monitoring",
)]
pub struct RequestsAction {
    app: Arc<AppContext>,
}

impl RequestsAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &RequestsAction,
    input_data: RequestActionInputContract,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let db_table =
        crate::db_operations::read::table::get(action.app.as_ref(), input_data.table_name.as_str())
            .await?;

    let src = db_table.request_metrics.get_metrics().await;

    let mut result = Vec::with_capacity(src.len());

    for metric in db_table.request_metrics.get_metrics().await {
        result.push(RequestContract::from(metric));
    }

    return Ok(HttpOutput::as_json(result).into_ok_result(true));
}
