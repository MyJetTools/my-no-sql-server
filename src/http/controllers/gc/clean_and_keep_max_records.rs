use std::sync::Arc;

use my_http_server::{
    middlewares::controllers::{
        actions::PostAction,
        documentation::{data_types::HttpDataType, out_results::HttpResult, HttpActionDescription},
    },
    HttpContext, HttpFailResult, HttpOkResult,
};

use crate::{app::AppContext, db_sync::EventSource};

use super::models::CleanPartitionAndKeepMaxRowsAmountInputContract;

pub struct CleanPartitionAndKepMaxRecordsControllerAction {
    app: Arc<AppContext>,
}

impl CleanPartitionAndKepMaxRecordsControllerAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl PostAction for CleanPartitionAndKepMaxRecordsControllerAction {
    fn get_route(&self) -> &str {
        "/GarbageCollector/CleanAndKeepMaxRecords"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: super::consts::CONTROLLER_NAME,
            description: "After operation some rows are going to be deleted to make sure we keep maximum rows amount required",

            input_params: CleanPartitionAndKeepMaxRowsAmountInputContract::get_input_params().into(),
            results: vec![HttpResult {
                http_code: 202,
                nullable: true,
                description: "Successful operation".to_string(),
                data_type: HttpDataType::None,
            }],
        }
        .into()
    }

    async fn handle_request(&self, ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let http_input =
            CleanPartitionAndKeepMaxRowsAmountInputContract::parse_http_input(ctx).await?;

        let db_table = crate::db_operations::read::table::get(
            self.app.as_ref(),
            http_input.table_name.as_str(),
        )
        .await?;

        let event_src = EventSource::as_client_request(self.app.as_ref(), http_input.sync_period);

        crate::db_operations::gc::clean_partition_and_keep_max_records::execute(
            self.app.as_ref(),
            db_table.as_ref(),
            http_input.partition_key.as_str(),
            http_input.max_amount,
            Some(event_src),
        )
        .await;

        HttpOkResult::Empty.into()
    }
}
