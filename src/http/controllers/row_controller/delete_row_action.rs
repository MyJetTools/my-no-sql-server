use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult};
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::db_operations;

use crate::app::AppContext;
use crate::db_sync::EventSource;

use super::models::{BaseDbRowContract, DeleteRowInputModel};

#[my_http_server_swagger::http_route(
    method: "DELETE",
    route: "/Row",
    controller: "Row",
    description: "Delete Entitiy",
    summary: "Delete Entitiy",
    input_data: "DeleteRowInputModel",
    result:[
        {status_code: 200, description: "Deleted row",  model:"BaseDbRowContract"},
    ]
)]
pub struct DeleteRowAction {
    app: Arc<AppContext>,
}

impl DeleteRowAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

/*
#[async_trait::async_trait]
impl DeleteAction for RowAction {
    fn get_route(&self) -> &str {
        "/Row"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: super::consts::CONTROLLER_NAME,
            description: "Delete Entitiy",

            input_params: DeleteRowInputModel::get_input_params().into(),
            results: vec![
                HttpResult {
                    http_code: 200,
                    nullable: false,
                    description: "Deleted row".to_string(),
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
    action: &DeleteRowAction,
    http_input: DeleteRowInputModel,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let db_table =
        crate::db_operations::read::table::get(action.app.as_ref(), http_input.table_name.as_ref())
            .await?;

    let event_src = EventSource::as_client_request(action.app.as_ref());

    let now = DateTimeAsMicroseconds::now();

    db_operations::write::delete_row::execute(
        action.app.as_ref(),
        &db_table,
        &http_input.partition_key,
        http_input.row_key.as_str(),
        event_src,
        http_input.sync_period.get_sync_moment(),
        now,
    )
    .await?
    .into()
}
