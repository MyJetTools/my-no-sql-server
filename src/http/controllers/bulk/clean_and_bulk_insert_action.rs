use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};
use my_no_sql_core::db_json_entity::JsonTimeStamp;

use crate::app::AppContext;
use crate::db_sync::EventSource;

use super::models::CleanAndBulkInsertInputContract;

#[my_http_server_swagger::http_route(
    method: "POST",
    route: "/Bulk/CleanAndBulkInsert",
    input_data: "CleanAndBulkInsertInputContract",
    summary: "Cleans partition and does bulk insert operation transactionally",
    description: "Cleans partition and does bulk insert operation transactionally",
    controller: "Bulk",
    result:[
        {status_code: 202, description: "Successful operation"},
        {status_code: 404, description: "Table not found"},
    ]
)]
pub struct CleanAndBulkInsertControllerAction {
    app: Arc<AppContext>,
}

impl CleanAndBulkInsertControllerAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &CleanAndBulkInsertControllerAction,
    input_data: CleanAndBulkInsertInputContract,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let db_table =
        crate::db_operations::read::table::get(action.app.as_ref(), input_data.table_name.as_str())
            .await?;

    let event_src = EventSource::as_client_request(action.app.as_ref());

    let now = JsonTimeStamp::now();

    let rows_by_partition = crate::db_operations::parse_json_entity::parse_as_btree_map(
        input_data.body.as_slice(),
        &now,
    )?;

    match &input_data.partition_key {
        Some(partition_key) => {
            crate::db_operations::write::clean_partition_and_bulk_insert::execute(
                action.app.as_ref(),
                db_table,
                partition_key,
                rows_by_partition,
                event_src,
                input_data.sync_period.get_sync_moment(),
            )
            .await?;
        }
        None => {
            crate::db_operations::write::clean_table_and_bulk_insert::execute(
                action.app.as_ref(),
                db_table,
                rows_by_partition,
                Some(event_src),
                input_data.sync_period.get_sync_moment(),
            )
            .await?;
        }
    }

    return HttpOutput::Empty.into_ok_result(true).into();
}
