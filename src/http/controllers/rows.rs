use crate::{
    app::AppServices,
    http::{http_ctx::HttpContext, http_fail::HttpFailResult, http_ok::HttpOkResult},
};

use super::consts;

pub async fn get_single_partition_multiple_rows(
    ctx: HttpContext,
    app: &AppServices,
) -> Result<HttpOkResult, HttpFailResult> {
    let query = ctx.get_query_string();
    let table_name = query.get_query_required_string_parameter(consts::PARAM_TABLE_NAME)?;

    let partition_key = query.get_query_required_string_parameter(consts::PARAM_PARTITION_KEY)?;

    let body = ctx.get_body().await;

    let db_table = app.get_table(table_name).await?;

    let row_keys = serde_json::from_slice(body.as_slice()).unwrap();

    let result = db_table
        .get_single_partition_multiple_rows(partition_key, row_keys)
        .await?;

    Ok(result.into())
}

pub async fn get_highest_row_and_below(
    ctx: HttpContext,
    app: &AppServices,
) -> Result<HttpOkResult, HttpFailResult> {
    let query = ctx.get_query_string();
    let table_name = query.get_query_required_string_parameter(consts::PARAM_TABLE_NAME)?;

    let partition_key = query.get_query_required_string_parameter(consts::PARAM_PARTITION_KEY)?;
    let row_key = query.get_query_required_string_parameter(consts::PARAM_ROW_KEY)?;

    let max_amount = query.get_query_required_parameter("maxAmount")?;

    let db_table = app.get_table(table_name).await?;

    let result = db_table
        .get_highest_row_and_below(partition_key, row_key.to_string(), max_amount)
        .await?;

    Ok(result.into())
}
