use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};
use my_http_server_controllers::controllers::{
    actions::DeleteAction,
    documentation::{data_types::HttpDataType, out_results::HttpResult, HttpActionDescription},
};

use crate::{app::AppContext, db_sync::EventSource};

use super::models::DeletePartitionsInputContract;

pub struct DeletePartitionsAction {
    app: Arc<AppContext>,
}

impl DeletePartitionsAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

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

    async fn handle_request(&self, ctx: &mut HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let input_data = DeletePartitionsInputContract::parse_http_input(ctx).await?;

        let db_table = crate::db_operations::read::table::get(
            self.app.as_ref(),
            input_data.table_name.as_ref(),
        )
        .await?;

        let event_src = EventSource::as_client_request(self.app.as_ref());

        crate::db_operations::write::delete_partitions(
            self.app.as_ref(),
            db_table.as_ref(),
            input_data.body.partition_keys,
            event_src,
            input_data.sync_period.get_sync_moment(),
        )
        .await;

        HttpOutput::Empty.into_ok_result(true).into()
    }
}
