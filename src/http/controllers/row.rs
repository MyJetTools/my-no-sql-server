use crate::db_operations::rows;
use crate::http::http_ctx::HttpContext;

use crate::app::AppServices;
use crate::http::http_fail::HttpFailResult;
use crate::http::http_helpers;
use crate::http::http_ok::HttpOkResult;

use super::consts;

pub async fn get_rows(ctx: HttpContext, app: &AppServices) -> Result<HttpOkResult, HttpFailResult> {
    let query = ctx.get_query_string();
    let table_name = query.get_query_required_string_parameter(consts::PARAM_TABLE_NAME)?;

    let partition_key = query.get_query_optional_string_parameter(consts::PARAM_PARTITION_KEY);
    let row_key = query.get_query_optional_string_parameter(consts::PARAM_ROW_KEY);

    let db_table = app.get_table(table_name).await?;

    let result = db_table.get_rows(partition_key, row_key).await?;

    Ok(result.into())
}

pub async fn insert(ctx: HttpContext, app: &AppServices) -> Result<HttpOkResult, HttpFailResult> {
    let query = ctx.get_query_string();
    let table_name = query.get_query_required_string_parameter(consts::PARAM_TABLE_NAME)?;

    let sync_period = query.get_sync_period();

    let body = ctx.get_body().await;

    let db_table = app.get_table(table_name).await?;

    let attr = http_helpers::create_transaction_attributes(app, sync_period);

    rows::insert(app, db_table, &body, Some(attr)).await?;

    Ok(HttpOkResult::Ok)
}

pub async fn insert_or_replace(
    ctx: HttpContext,
    app: &AppServices,
) -> Result<HttpOkResult, HttpFailResult> {
    let query = ctx.get_query_string();
    let table_name = query.get_query_required_string_parameter(consts::PARAM_TABLE_NAME)?;

    let sync_period = query.get_sync_period();

    let body = ctx.get_body().await;

    let db_table = app.get_table(table_name).await?;

    let attr = http_helpers::create_transaction_attributes(app, sync_period);

    rows::insert_or_replace(app, db_table, &body, Some(attr)).await?;

    Ok(HttpOkResult::Ok)
}

pub async fn replace(ctx: HttpContext, app: &AppServices) -> Result<HttpOkResult, HttpFailResult> {
    let query = ctx.get_query_string();
    let table_name = query.get_query_required_string_parameter(consts::PARAM_TABLE_NAME)?;

    let sync_period = query.get_sync_period();

    let body = ctx.get_body().await;

    let db_table = app.get_table(table_name).await?;

    let attr = http_helpers::create_transaction_attributes(app, sync_period);

    rows::replace(app, db_table, body.as_slice(), Some(attr)).await?;

    Ok(HttpOkResult::Ok)
}
