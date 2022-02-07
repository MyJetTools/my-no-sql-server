use crate::app::AppContext;
use async_trait::async_trait;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};
use my_http_server_controllers::controllers::{
    actions::PostAction,
    documentation::{out_results::HttpResult, HttpActionDescription},
};
use std::sync::Arc;

use super::models::StartTransactionResponse;

pub struct StartTransactionAction {
    app: Arc<AppContext>,
}

impl StartTransactionAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait]
impl PostAction for StartTransactionAction {
    fn get_route(&self) -> &str {
        "/Transactions/Start"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: super::consts::CONTROLLER_NAME,
            description: "Start new Transaction",

            input_params: None,
            results: vec![HttpResult {
                http_code: 200,
                nullable: true,
                description: "Issued transaction".to_string(),
                data_type: StartTransactionResponse::get_http_data_structure()
                    .into_http_data_type_object(),
            }],
        }
        .into()
    }

    async fn handle_request(&self, _: &mut HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let transaction_id = self.app.active_transactions.issue_new().await;

        let response = StartTransactionResponse { transaction_id };

        HttpOutput::as_json(response).into_ok_result(true).into()
    }
}
