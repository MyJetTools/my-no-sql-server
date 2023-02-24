use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult};

use my_no_sql_core::db_json_entity::JsonTimeStamp;

use crate::app::AppContext;
use crate::db_sync::EventSource;

use super::models::{BaseDbRowContract, ReplaceInputContract};

#[my_http_server_swagger::http_route(
    method: "PUT",
    route: "/Row/Replace",
    controller: "Row",
    description: "Replace Entitiy",
    summary: "Replaces Entitiy",
    input_data: "ReplaceInputContract",
    result:[
        {status_code: 200, description: "Replaced row",  model:"BaseDbRowContract"},
    ]
)]
pub struct ReplaceRowAction {
    app: Arc<AppContext>,
}

impl ReplaceRowAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

/*
#[async_trait::async_trait]
impl PutAction for RowAction {
    fn get_route(&self) -> &str {
        "/Row/Replace"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: super::consts::CONTROLLER_NAME,
            description: "Replace Entitiy",

            input_params: DeleteRowInputModel::get_input_params().into(),
            results: vec![
                HttpResult {
                    http_code: 200,
                    nullable: false,
                    description: "Replaced row".to_string(),
                    data_type: BaseDbRowContract::get_http_data_structure()
                        .into_http_data_type_object(),
                },
                crate::http::docs::rejects::op_with_table_is_failed(),
            ],
        }
        .into()
    }
}
 */
async fn handle_request(
    action: &ReplaceRowAction,
    input_data: ReplaceInputContract,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let db_table =
        crate::db_operations::read::table::get(action.app.as_ref(), input_data.table_name.as_ref())
            .await?;

    let now = JsonTimeStamp::now();

    let db_json_entity =
        crate::db_operations::parse_json_entity::as_single_entity(input_data.body.as_slice())?;

    crate::db_operations::write::replace::validate_before(
        action.app.as_ref(),
        &db_table,
        db_json_entity.partition_key,
        db_json_entity.row_key,
        db_json_entity.time_stamp,
    )
    .await?;

    let db_row = Arc::new(db_json_entity.new_db_row(&now));

    let event_src = EventSource::as_client_request(action.app.as_ref());

    crate::db_operations::write::replace::execute(
        action.app.as_ref(),
        &db_table,
        db_json_entity.partition_key,
        db_row,
        event_src,
        db_json_entity.time_stamp.unwrap(),
        input_data.sync_period.get_sync_moment(),
    )
    .await?
    .into()
}
