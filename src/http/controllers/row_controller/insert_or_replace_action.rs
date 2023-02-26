use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult};

use my_no_sql_core::db_json_entity::JsonTimeStamp;

use crate::app::AppContext;
use crate::db_sync::EventSource;

use super::models::InsertOrReplaceInputContract;

#[my_http_server_swagger::http_route(
    method: "POST",
    route: "/Row/InsertOrReplace",
    controller: "Row",
    description: "Insert or replace DbEntity",
    summary: "Inserts or replaces DbEntity",
    input_data: "InsertOrReplaceInputContract",
    result:[
        {status_code: 200, description: "Removed entities"},
    ]
)]
pub struct InsertOrReplaceAction {
    app: Arc<AppContext>,
}

impl InsertOrReplaceAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &InsertOrReplaceAction,
    input_data: InsertOrReplaceInputContract,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let db_table =
        crate::db_operations::read::table::get(action.app.as_ref(), input_data.table_name.as_str())
            .await?;

    let event_src = EventSource::as_client_request(action.app.as_ref());

    let now = JsonTimeStamp::now();

    let db_json_entity =
        crate::db_operations::parse_json_entity::as_single_entity(input_data.body.as_slice())?;

    let db_row = Arc::new(db_json_entity.new_db_row(&now));

    crate::db_operations::write::insert_or_replace::execute(
        action.app.as_ref(),
        db_table,
        db_row,
        event_src,
        input_data.sync_period.get_sync_moment(),
        now.date_time,
    )
    .await?
    .into()
}
