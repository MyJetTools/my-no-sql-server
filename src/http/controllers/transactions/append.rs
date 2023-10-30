use crate::app::AppContext;
use my_http_server::macros::*;
use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};

use super::models::ProcessTransactionInputModel;

#[http_route(
    method: "POST",
    route: "/Transactions/Append",
    description: "Get Table size",
    summary: "Returns Table size",
    input_data: "ProcessTransactionInputModel",
    controller: "Transactions",
    result:[
        {status_code: 202, description: "Actions are added suscessfully"},        
    ]
)]
pub struct AppendTransactionAction {
    app: Arc<AppContext>,
}

impl AppendTransactionAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

/*
#[async_trait]
impl PostAction for AppendTransactionAction {
    fn get_route(&self) -> &str {
        "/Transactions/Append"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: super::consts::CONTROLLER_NAME,
            description: "Append actions to transaction",

            input_params: Some(ProcessTransactionInputModel::get_input_params()),
            results: vec![
                response::empty("Actions are added suscessfully"),
                super::models::transaction_not_found_response_doc(),
            ],
        }
        .into()
    }
}
 */
async fn handle_request(
    action: &AppendTransactionAction,
    input_model: ProcessTransactionInputModel,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let tranactions =
        crate::db_transactions::json_parser::parse_transactions(input_model.body.as_slice())?;

    crate::db_operations::transactions::append_events(
        action.app.as_ref(),
        input_model.transaction_id.as_str(),
        tranactions,
    )
    .await?;

    HttpOutput::Empty.into_ok_result(true)
}
