use std::{collections::HashMap, sync::Arc};

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};

use crate::db_json_entity::JsonTimeStamp;

use crate::app::AppContext;
use crate::db_sync::EventSource;

use super::models::BulkDeleteInputContract;

#[my_http_server_swagger::http_route(
    method: "POST",
    route: "/Bulk/Delete",
    input_data: "BulkDeleteInputContract",
    description: "Bulk delete operation",
    controller: "Bulk",
    result:[
        {status_code: 202, description: "Successful operation"},
        {status_code: 404, description: "Table not found"},
    ]
)]
pub struct BulkDeleteControllerAction {
    app: Arc<AppContext>,
}

impl BulkDeleteControllerAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &BulkDeleteControllerAction,
    input_data: BulkDeleteInputContract,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let db_table =
        crate::db_operations::read::table::get(action.app.as_ref(), input_data.table_name.as_str())
            .await?;

    let event_src = EventSource::as_client_request(action.app.as_ref());

    let rows_to_delete: HashMap<String, Vec<String>> =
        serde_json::from_slice(input_data.body.as_slice()).unwrap();

    let now = JsonTimeStamp::now();

    crate::db_operations::write::bulk_delete(
        action.app.as_ref(),
        db_table.as_ref(),
        rows_to_delete,
        event_src,
        now.date_time,
        input_data.sync_period.get_sync_moment(),
    )
    .await?;

    HttpOutput::Empty.into_ok_result(true).into()
}
