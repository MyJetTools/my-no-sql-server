use crate::{app::AppContext, db_sync::EventSource};
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};

use std::sync::Arc;

use super::models::ProcessTransactionInputModel;

#[my_http_server_swagger::http_route(
    method: "POST",
    route: "/Transactions/Commit",
    description: "Commit transaction",
    summary: "Commits transaction",
    input_data: "ProcessTransactionInputModel",
    controller: "Transactions",
    result:[
        {status_code: 202, description: "Transaction is canceled"},        
    ]
)]
pub struct CommitTransactionAction {
    app: Arc<AppContext>,
}

impl CommitTransactionAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

/*
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
}
 */

async fn handle_request(
    action: &CommitTransactionAction,
    input_model: ProcessTransactionInputModel,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let even_src = EventSource::as_client_request(action.app.as_ref());

    crate::db_operations::transactions::commit(
        action.app.as_ref(),
        input_model.transaction_id.as_ref(),
        even_src,
        crate::db_sync::DataSynchronizationPeriod::Sec1.get_sync_moment(),
    )
    .await?;

    return HttpOutput::Empty.into_ok_result(true).into();
}
