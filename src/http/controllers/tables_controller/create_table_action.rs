use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};

use crate::{app::AppContext, db_sync::EventSource};

use super::models::CreateTableCotnract;

#[my_http_server_swagger::http_route(
    method: "POST",
    route: "/Tables/Create",
    input_data: "CreateTableCotnract",
    description: "Create table",
    summary: "Creates table",
    controller: "Tables",
    result:[
        {status_code: 202, description: "Table is created"},
        {status_code: 400, description: "Table already exists"},
    ]
)]
pub struct CreateTableAction {
    app: Arc<AppContext>,
}

impl CreateTableAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &CreateTableAction,
    input_data: CreateTableCotnract,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let even_src = EventSource::as_client_request(action.app.as_ref());

    crate::db_operations::write::table::create(
        action.app.as_ref(),
        input_data.table_name.as_str(),
        input_data.persist,
        input_data.max_partitions_amount,
        input_data.max_rows_per_partition_amount,
        even_src,
        input_data.sync_period.get_sync_moment(),
    )
    .await?;

    return HttpOutput::Empty.into_ok_result(true);
}
