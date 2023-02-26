use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};

use crate::{app::AppContext, db_sync::EventSource};

use super::models::DeletePartitionsInputContract;

#[my_http_server_swagger::http_route(
    method: "DELETE",
    route: "/Rows/DeletePartitions",
    controller: "Rows",
    description: "Delete Partitions",
    summary: "Deletes Partitions",
    input_data: "DeletePartitionsInputContract",
    result:[
        {status_code: 200, description: "Removed entities"},
    ]
)]
pub struct DeletePartitionsAction {
    app: Arc<AppContext>,
}

impl DeletePartitionsAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

/*
#[async_trait::async_trait]
impl DeleteAction for DeletePartitionsAction {
    fn get_route(&self) -> &str {
        "/Rows/DeletePartitions"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: super::consts::CONTROLLER_NAME,
            description: "Delete Partitions",

            input_params: DeletePartitionsInputContract::get_input_params().into(),
            results: vec![
                HttpResult {
                    http_code: 202,
                    nullable: false,
                    description: "Rows".to_string(),
                    data_type: HttpDataType::None,
                },
                crate::http::docs::rejects::op_with_table_is_failed(),
            ],
        }
        .into()
    }


}
 */

async fn handle_request(
    action: &DeletePartitionsAction,
    input_data: DeletePartitionsInputContract,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let db_table =
        crate::db_operations::read::table::get(action.app.as_ref(), input_data.table_name.as_ref())
            .await?;

    let event_src = EventSource::as_client_request(action.app.as_ref());

    let partition_keys = input_data.body.deserialize_json()?;

    crate::db_operations::write::delete_partitions(
        action.app.as_ref(),
        &db_table,
        partition_keys.partition_keys.into_iter(),
        event_src,
        input_data.sync_period.get_sync_moment(),
    )
    .await?;

    HttpOutput::Empty.into_ok_result(true).into()
}
