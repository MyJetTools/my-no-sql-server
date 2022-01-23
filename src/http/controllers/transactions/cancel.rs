use std::sync::Arc;

use my_http_server::{
    middlewares::controllers::{
        actions::PostAction,
        documentation::{data_types::HttpObjectStructure, HttpActionDescription},
    },
    HttpContext, HttpFailResult, HttpOkResult,
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
    fn get_additional_types(&self) -> Option<Vec<HttpObjectStructure>> {
        None
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

    async fn handle_request(&self, ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let input_model: ProcessTransactionInputModel =
            ProcessTransactionInputModel::parse_http_input(ctx).await?;

        crate::db_operations::transactions::cancel(
            self.app.as_ref(),
            input_model.transaction_id.as_str(),
        )
        .await?;
        return Ok(HttpOkResult::Empty);
    }
}
