use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};

use crate::{app::AppContext, db_sync::EventSource};

use super::models::CleanTableContract;

#[my_http_server_swagger::http_route(
    method: "PUT",
    route: "/Tables/Clean",
    input_data: "CleanTableContract",
    description: "Clean Table",
    controller: "Tables",
    result:[
        {status_code: 202, description: "Table is cleaned"},
    ]
)]
pub struct CleanTableAction {
    app: Arc<AppContext>,
}

impl CleanTableAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}
async fn handle_request(
    action: &CleanTableAction,
    input_data: CleanTableContract,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let db_table =
        crate::db_operations::read::table::get(action.app.as_ref(), input_data.table_name.as_ref())
            .await?;

    let event_src = EventSource::as_client_request(action.app.as_ref());

    crate::db_operations::write::clean_table::execute(
        action.app.as_ref(),
        db_table,
        event_src,
        input_data.sync_period.get_sync_moment(),
    )
    .await?;

    return Ok(HttpOutput::Empty.into_ok_result(true));
}
