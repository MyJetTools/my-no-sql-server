use async_trait::async_trait;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};
use my_http_server_controllers::controllers::{
    actions::GetAction,
    documentation::{data_types::HttpDataType, out_results::HttpResult, HttpActionDescription},
};

use super::{super::super::contracts::response, models::GetPartitionsAmountContract};
use crate::app::AppContext;
use std::{result::Result, sync::Arc};

pub struct GetPartitionsCountAction {
    app: Arc<AppContext>,
}

impl GetPartitionsCountAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait]
impl GetAction for GetPartitionsCountAction {
    fn get_route(&self) -> &str {
        "/Tables/PartitionsCount"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: super::consts::CONTROLLER_NAME,
            description: "Get Partitions count",

            input_params: GetPartitionsAmountContract::get_input_params().into(),
            results: vec![
                HttpResult {
                    http_code: 200,
                    nullable: true,
                    description: "Partitions count".to_string(),
                    data_type: HttpDataType::as_long(),
                },
                response::table_not_found(),
            ],
        }
        .into()
    }

    async fn handle_request(&self, ctx: &mut HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let input_data = GetPartitionsAmountContract::parse_http_input(ctx).await?;

        let db_table = crate::db_operations::read::table::get(
            self.app.as_ref(),
            input_data.table_name.as_str(),
        )
        .await?;

        let partitions_amount = db_table.get_partitions_amount().await;

        HttpOutput::as_text(format!("{}", partitions_amount))
            .into_ok_result(true)
            .into()
    }
}
