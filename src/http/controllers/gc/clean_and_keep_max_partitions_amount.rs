use std::sync::Arc;

use my_http_server::{
    middlewares::controllers::{
        actions::PostAction,
        documentation::{data_types::HttpDataType, out_results::HttpResult, HttpActionDescription},
    },
    HttpContext, HttpFailResult, HttpOkResult,
};

use crate::app::AppContext;

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

    async fn handle_request(&self, ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let input_data =
            CleanAndKeepMaxPartitionsAmountInputContract::parse_http_input(ctx).await?;

        let db_table = crate::db_operations::read::table::get(
            self.app.as_ref(),
            input_data.table_name.as_str(),
        )
        .await?;

        let attr = crate::operations::transaction_attributes::create(
            self.app.as_ref(),
            input_data.sync_period,
        );

        crate::db_operations::gc::keep_max_partitions_amount::execute(
            self.app.as_ref(),
            db_table,
            input_data.max_partitions_amount,
            Some(attr),
        )
        .await;

        HttpOkResult::Empty.into()
    }
}
