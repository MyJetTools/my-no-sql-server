use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};
use my_no_sql_core::db_json_entity::JsonTimeStamp;

use crate::app::AppContext;
use crate::db_sync::EventSource;

use super::models::InsertInputContract;

#[my_http_server_swagger::http_route(
    method: "POST",
    route: "/Row/Insert",
    controller: "Row",
    description: "Insert Row",
    summary: "Inserts Row",
    input_data: "InsertInputContract",
    result:[
        {status_code: 200, description: "Amount of rows of the table or the partition"},
    ]
)]
pub struct InsertRowAction {
    app: Arc<AppContext>,
}

impl InsertRowAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &InsertRowAction,
    input_data: InsertInputContract,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    // let input_data = InsertInputContract::parse_http_input(ctx).await?;

    let db_table =
        crate::db_operations::read::table::get(action.app.as_ref(), input_data.table_name.as_str())
            .await?;

    let db_json_entity =
        crate::db_operations::parse_json_entity::as_single_entity(input_data.body.as_slice())?;

    crate::db_operations::write::insert::validate_before(
        action.app.as_ref(),
        &db_table,
        db_json_entity.partition_key,
        db_json_entity.row_key,
    )
    .await?;

    let event_src = EventSource::as_client_request(action.app.as_ref());

    let now = JsonTimeStamp::now();

    let db_row = Arc::new(db_json_entity.new_db_row(&now));

    crate::db_operations::write::insert::execute(
        &action.app,
        db_table,
        db_row,
        event_src,
        input_data.sync_period.get_sync_moment(),
        now.date_time,
    )
    .await?;

    HttpOutput::Empty.into_ok_result(true).into()
}
