use crate::{app::AppContext, http::http_ok::HttpOkResult};

use my_http_utils::HttpFailResult;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct StartTransactionResponse {
    #[serde(rename = "transactionId")]
    transaction_id: String,
}

pub async fn post(app: &AppContext) -> Result<HttpOkResult, HttpFailResult> {
    let transaction_id = app.active_transactions.issue_new().await;

    let response = StartTransactionResponse { transaction_id };

    return HttpOkResult::create_json_response(response);
}
