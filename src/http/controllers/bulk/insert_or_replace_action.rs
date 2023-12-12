use my_http_server::macros::*;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};
use my_no_sql_sdk::core::db_json_entity::JsonTimeStamp;
use std::sync::Arc;

use crate::app::AppContext;
use crate::db_sync::EventSource;

use super::models::BulkInsertOrReplaceInputContract;

#[http_route(
    method: "POST",
    route: "/api/Bulk/InsertOrReplace",
    deprecated_routes: ["/Bulk/InsertOrReplace"],
    input_data: "BulkInsertOrReplaceInputContract",

    summary: "Bulk insert or replace operation",
    description: "Executes Bulk insert or replace operation",
    controller: "Bulk",
    result:[
        {status_code: 202, description: "Successful operation"},
        {status_code: 404, description: "Table not found"},
    ]
)]
pub struct BlukInsertOrReplaceControllerAction {
    app: Arc<AppContext>,
}

impl BlukInsertOrReplaceControllerAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &BlukInsertOrReplaceControllerAction,
    input_data: BulkInsertOrReplaceInputContract,
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

    crate::db_operations::write::bulk_insert_or_update::execute(
        action.app.as_ref(),
        &db_table,
        rows_by_partition,
        event_src,
        input_data.sync_period.get_sync_moment(),
        now.date_time,
    )
    .await?;

    HttpOutput::Empty.into_ok_result(true).into()
}

/*
#[async_trait::async_trait]
impl PostAction for BlukInsertOrReplaceControllerAction {
    fn get_route(&self) -> &str {
        "/Bulk/InsertOrReplace"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: super::consts::CONTROLLER_NAME,
            description: "Bulk insert or replace operation",

            input_params: BulkInsertOrReplaceInputContract::get_input_params().into(),
            results: vec![HttpResult {
                http_code: 202,
                nullable: true,
                description: "Successful operation".to_string(),
                data_type: HttpDataType::None,
            }],
        }
        .into()
    }
}
 */
