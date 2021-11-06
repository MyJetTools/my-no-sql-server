use crate::{
    app::AppContext,
    http::{http_ctx::HttpContext, http_helpers, http_ok::HttpOkResult},
};

use my_http_utils::HttpFailResult;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct StartTransactionResponse {
    #[serde(rename = "transactionId")]
    transaction_id: String,
}

pub async fn start(app: &AppContext) -> Result<HttpOkResult, HttpFailResult> {
    let transaction_id = app.active_transactions.issue_new().await;

    let response = StartTransactionResponse { transaction_id };

    return HttpOkResult::create_json_response(response);
}

pub async fn append(app: &AppContext, ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult> {
    let query_string = ctx.get_query_string()?;

    let transaction_id = query_string.get_query_required_string_parameter("transactionId")?;

    let body = ctx.get_body().await;

    let tranactions = crate::db_transactions::json_parser::parse_transactions(body.as_slice())?;

    crate::db_operations::transactions::append_events(app, transaction_id, tranactions).await?;

    return Ok(HttpOkResult::Ok);
}

pub async fn commit(app: &AppContext, ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult> {
    let query_string = ctx.get_query_string()?;

    let transaction_id = query_string.get_query_required_string_parameter("transactionId")?;

    let attr = http_helpers::create_transaction_attributes(
        app,
        crate::db_sync::DataSynchronizationPeriod::Sec1,
    );

    crate::db_operations::transactions::commit(app, transaction_id, attr).await?;

    return Ok(HttpOkResult::Ok);
}

pub async fn cancel(app: &AppContext, ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult> {
    let query_string = ctx.get_query_string()?;
    let transaction_id = query_string.get_query_required_string_parameter("transactionId")?;
    crate::db_operations::transactions::cancel(app, transaction_id).await?;
    return Ok(HttpOkResult::Ok);
}
