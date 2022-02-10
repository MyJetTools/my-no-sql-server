use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};
use my_http_server_controllers::controllers::{
    actions::PostAction, documentation::HttpActionDescription,
};

use crate::{app::AppContext, http::contracts::response};

use super::models::ProcessTransactionInputModel;

pub struct CancelTransactionAction {
    app: Arc<AppContext>,
}

impl CancelTransactionAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl PostAction for CancelTransactionAction {
    fn get_route(&self) -> &str {
        "/Transactions/Cancel"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: super::consts::CONTROLLER_NAME,
            description: "Cancel transaction",

            input_params: Some(ProcessTransactionInputModel::get_input_params()),
            results: vec![
                response::empty("Transaction is canceled"),
                super::models::transaction_not_found_response_doc(),
            ],
        }
        .into()
    }

    async fn handle_request(&self, ctx: &mut HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let input_model: ProcessTransactionInputModel =
            ProcessTransactionInputModel::parse_http_input(ctx).await?;

        crate::db_operations::transactions::cancel(
            self.app.as_ref(),
            input_model.transaction_id.as_str(),
        )
        .await?;
        return Ok(HttpOutput::Empty.into_ok_result(true));
    }
}
