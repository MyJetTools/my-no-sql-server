use crate::{
    app::AppContext,
    http::{http_ctx::HttpContext, http_ok::HttpOkResult},
};

use my_http_utils::HttpFailResult;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct StartTransactionResponse {
    #[serde(rename = "transactionId")]
    transaction_id: String,
}

pub async fn post(app: &AppContext, ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult> {
    let query_string = ctx.get_query_string()?;

    let transaction_id = query_string.get_query_required_string_parameter("transactionId")?;

    let body = ctx.get_body().await;

    let tranactions = crate::db_transactions::json_parser::parse_transactions(body.as_slice())?;

    crate::db_operations::transactions::append_events(app, transaction_id, tranactions).await?;

    return Ok(HttpOkResult::Empty);
}
