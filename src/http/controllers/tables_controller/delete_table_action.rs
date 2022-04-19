use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};

use crate::{app::AppContext, db_sync::EventSource};

use super::{super::super::contracts::input_params::*, models::DeleteTableContract};

#[my_http_server_swagger::http_route(
    method: "DELETE",
    route: "/Tables/Delete",
    input_data: "DeleteTableContract",
    description: "Delete Table",
    controller: "Tables",
    result:[
        {status_code: 202, description: "Table is deleted"},
    ]
)]
pub struct DeleteTableAction {
    app: Arc<AppContext>,
}

impl DeleteTableAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &DeleteTableAction,
    input_data: DeleteTableContract,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    if input_data.api_key != action.app.settings.table_api_key.as_str() {
        return Err(HttpFailResult::as_unauthorized(None));
    }

    let event_src = EventSource::as_client_request(action.app.as_ref());

    crate::db_operations::write::table::delete(
        action.app.clone(),
        input_data.table_name,
        event_src,
        DEFAULT_SYNC_PERIOD.get_sync_moment(),
    )
    .await?;

    return Ok(HttpOutput::Empty.into_ok_result(true));
}
