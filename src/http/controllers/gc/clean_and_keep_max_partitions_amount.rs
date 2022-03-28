use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};
use my_http_server_controllers::controllers::{
    actions::PostAction,
    documentation::{data_types::HttpDataType, out_results::HttpResult, HttpActionDescription},
};

use crate::{app::AppContext, db_sync::EventSource};

use super::models::CleanAndKeepMaxPartitionsAmountInputContract;

pub struct CleanAndKeepMaxPartitionsAmount {
    app: Arc<AppContext>,
}

impl CleanAndKeepMaxPartitionsAmount {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl PostAction for CleanAndKeepMaxPartitionsAmount {
    fn get_route(&self) -> &str {
        "/GarbageCollector/CleanAndKeepMaxPartitions"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: super::consts::CONTROLLER_NAME,
            description: "After operation some partitions can be deleted to make sure we keep maximum partitions amount required",

            input_params: CleanAndKeepMaxPartitionsAmountInputContract::get_input_params().into(),
            results: vec![HttpResult {
                http_code: 202,
                nullable: true,
                description: "Successful operation".to_string(),
                data_type: HttpDataType::None,
            }],
        }
        .into()
    }

    async fn handle_request(&self, ctx: &mut HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let http_input =
            CleanAndKeepMaxPartitionsAmountInputContract::parse_http_input(ctx).await?;

        let db_table = crate::db_operations::read::table::get(
            self.app.as_ref(),
            http_input.table_name.as_str(),
        )
        .await?;

        let event_src = EventSource::as_client_request(self.app.as_ref());

        crate::db_operations::gc::keep_max_partitions_amount(
            self.app.as_ref(),
            db_table,
            http_input.max_partitions_amount,
            event_src,
            http_input.sync_period.get_sync_moment(),
        )
        .await;

        HttpOutput::Empty.into_ok_result(true).into()
    }
}
