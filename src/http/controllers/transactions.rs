use crate::{
    app::AppServices,
    db::{FailOperationResult, OperationResult},
    http::{http_ctx::HttpContext, http_helpers},
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct StartTransactionResponse {
    #[serde(rename = "transactionId")]
    transaction_id: String,
}

pub async fn start(app: &AppServices) -> Result<OperationResult, FailOperationResult> {
    let transaction_id = app.active_transactions.issue_new().await;

    let response = StartTransactionResponse { transaction_id };

    return OperationResult::create_json_response(response);
}

pub async fn append(
    app: &AppServices,
    ctx: HttpContext,
) -> Result<OperationResult, FailOperationResult> {
    let query_string = ctx.get_query_string();

    let transaction_id = query_string.get_query_required_string_parameter("transactionId")?;

    let body = ctx.get_body().await;

    crate::db_transactional_operations::http::appen_events(app, transaction_id, body).await?;

    return Ok(OperationResult::Ok);
}

pub async fn commit(
    app: &AppServices,
    ctx: HttpContext,
) -> Result<OperationResult, FailOperationResult> {
    let query_string = ctx.get_query_string();

    let transaction_id = query_string.get_query_required_string_parameter("transactionId")?;

    let attr = http_helpers::create_transaction_attributes(
        app,
        crate::db_transactions::DataSynchronizationPeriod::Sec1,
    );

    crate::db_transactional_operations::http::commit(app, transaction_id, attr).await?;

    return Ok(OperationResult::Ok);
}

pub async fn cancel(
    app: &AppServices,
    ctx: HttpContext,
) -> Result<OperationResult, FailOperationResult> {
    let query_string = ctx.get_query_string();

    let transaction_id = query_string.get_query_required_string_parameter("transactionId")?;

    let result = app.active_transactions.remove(transaction_id).await;

    match result {
        Some(_) => Ok(OperationResult::Ok),
        None => Err(FailOperationResult::TransactionNotFound {
            id: transaction_id.to_string(),
        }),
    }
}
