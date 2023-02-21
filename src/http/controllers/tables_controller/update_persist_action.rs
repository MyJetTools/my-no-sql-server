use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};

use super::models::UpdatePersistTableContract;
use crate::{app::AppContext, db_sync::EventSource};
use std::{result::Result, sync::Arc};

#[my_http_server_swagger::http_route(
    method: "POST",
    route: "/Tables/UpdatePersist",
    input_data: "UpdatePersistTableContract",
    description: "Update table persistence state",
    summary: "Updates table persistence state",
    controller: "Tables",
    result:[
        {status_code: 202, description: "Updated succesfully"},
        {status_code: 400, description: "Table not found"},
    ]
)]
pub struct UpdatePersistAction {
    app: Arc<AppContext>,
}

impl UpdatePersistAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &UpdatePersistAction,
    input_data: UpdatePersistTableContract,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let db_table =
        crate::db_operations::read::table::get(action.app.as_ref(), input_data.table_name.as_str())
            .await?;

    let event_src = EventSource::as_client_request(action.app.as_ref());

    crate::db_operations::write::table::update_persist_state(
        &action.app,
        db_table,
        input_data.persist,
        event_src,
    )
    .await?;

    HttpOutput::Empty.into_ok_result(true).into()
}
