use crate::{
    app::AppContext, db_json_entity::JsonTimeStamp, db_sync::EventSource, http::contracts::response,
};
use async_trait::async_trait;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};
use my_http_server_controllers::controllers::{
    actions::PostAction, documentation::HttpActionDescription,
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
    fn get_route(&self) -> &str {
        "/Transactions/Commit"
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

    async fn handle_request(&self, ctx: &mut HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let input_model: ProcessTransactionInputModel =
            ProcessTransactionInputModel::parse_http_input(ctx).await?;

        let even_src = EventSource::as_client_request(self.app.as_ref());

        let now = JsonTimeStamp::now();

        crate::db_operations::transactions::commit(
            self.app.as_ref(),
            input_model.transaction_id.as_ref(),
            even_src,
            &now,
            crate::db_sync::DataSynchronizationPeriod::Sec1.get_sync_moment(),
        )
        .await?;

        return HttpOutput::Empty.into_ok_result(true).into();
    }
}
