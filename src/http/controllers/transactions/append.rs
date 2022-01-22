use std::sync::Arc;

use crate::{app::AppContext, http::contracts::response};
use async_trait::async_trait;
use my_http_server::{
    middlewares::controllers::{
        actions::PostAction,
        documentation::{data_types::HttpObjectStructure, HttpActionDescription},
    },
    HttpContext, HttpFailResult, HttpOkResult,
};

use super::models::ProcessTransactionInputModel;

pub struct AppendTransactionAction {
    app: Arc<AppContext>,
}

impl AppendTransactionAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait]
impl PostAction for AppendTransactionAction {
    fn get_additional_types(&self) -> Option<Vec<HttpObjectStructure>> {
        None
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: super::consts::CONTROLLER_NAME,
            description: "Append actions to transaction",

            input_params: Some(ProcessTransactionInputModel::get_doc()),
            results: vec![
                response::empty("Actions are added suscessfully"),
                super::models::transaction_not_found_response_doc(),
            ],
        }
        .into()
    }

    async fn handle_request(&self, ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let input_model: ProcessTransactionInputModel =
            ProcessTransactionInputModel::parse_http_input(ctx).await?;

        let tranactions =
            crate::db_transactions::json_parser::parse_transactions(input_model.body.as_slice())?;

        crate::db_operations::transactions::append_events(
            self.app.as_ref(),
            input_model.transaction_id.as_str(),
            tranactions,
        )
        .await?;

        return Ok(HttpOkResult::Empty);
    }
}
