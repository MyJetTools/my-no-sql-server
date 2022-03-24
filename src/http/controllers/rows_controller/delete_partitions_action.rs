use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult};
use my_http_server_controllers::controllers::{
    actions::DeleteAction,
    documentation::{data_types::HttpDataType, out_results::HttpResult, HttpActionDescription},
};

use crate::app::AppContext;

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
        let data = DeletePartitionsInputContract::parse_http_input(ctx).await?;

        todo!("Implement")
    }
}
