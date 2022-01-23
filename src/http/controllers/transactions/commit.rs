use crate::{app::AppContext, db_json_entity::JsonTimeStamp, http::contracts::response};
use async_trait::async_trait;
use my_http_server::{
    middlewares::controllers::{
        actions::PostAction,
        documentation::{data_types::HttpObjectStructure, HttpActionDescription},
    },
    HttpContext, HttpFailResult, HttpOkResult,
};
use std::sync::Arc;

use super::models::ProcessTransactionInputModel;

pub struct CommitTransactionAction {
    app: Arc<AppContext>,
}

impl CommitTransactionAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait]
impl PostAction for CommitTransactionAction {
    fn get_additional_types(&self) -> Option<Vec<HttpObjectStructure>> {
        None
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: super::consts::CONTROLLER_NAME,
            description: "Commit transaction",

            input_params: Some(ProcessTransactionInputModel::get_input_params()),
            results: vec![
                response::empty("Transaction commited succesfully"),
                super::models::transaction_not_found_response_doc(),
            ],
        }
        .into()
    }

    async fn handle_request(&self, ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let input_model: ProcessTransactionInputModel =
            ProcessTransactionInputModel::parse_http_input(ctx).await?;

        let attr = crate::operations::transaction_attributes::create(
            self.app.as_ref(),
            crate::db_sync::DataSynchronizationPeriod::Sec1,
        );

        let now = JsonTimeStamp::now();

        crate::db_operations::transactions::commit(
            self.app.as_ref(),
            input_model.transaction_id.as_ref(),
            attr,
            &now,
        )
        .await?;

        return HttpOkResult::Empty.into();
    }
}
