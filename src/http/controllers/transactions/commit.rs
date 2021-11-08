use crate::{
    app::AppContext,
    db_json_entity::JsonTimeStamp,
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

    let attr = crate::operations::transaction_attributes::create(
        app,
        crate::db_sync::DataSynchronizationPeriod::Sec1,
    );

    let now = JsonTimeStamp::now();

    crate::db_operations::transactions::commit(app, transaction_id, attr, &now).await?;

    return Ok(HttpOkResult::Empty);
}
