use crate::db_operations::rows;
use crate::http::http_ctx::HttpContext;

use crate::app::AppServices;
use crate::http::http_fail::HttpFailResult;
use crate::http::http_helpers;
use crate::http::http_ok::HttpOkResult;

use super::consts;

pub async fn insert_or_replace(
    ctx: HttpContext,
    app: &AppServices,
) -> Result<HttpOkResult, HttpFailResult> {
    let query = ctx.get_query_string();
    let table_name = query.get_query_required_string_parameter(consts::PARAM_TABLE_NAME)?;

    let body = ctx.get_body().await;

    let db_table = app.get_table(table_name).await?;
    let sync_period = query.get_sync_period();

    let attr = http_helpers::create_transaction_attributes(app, sync_period);

    rows::bulk_insert_or_update(app, db_table, body.as_slice(), Some(attr)).await?;

    return Ok(HttpOkResult::Ok);
}

pub async fn clean_and_bulk_insert(
    ctx: HttpContext,
    app: &AppServices,
) -> Result<HttpOkResult, HttpFailResult> {
    let query = ctx.get_query_string();
    let table_name = query.get_query_required_string_parameter(consts::PARAM_TABLE_NAME)?;

    let partition_key_param =
        query.get_query_optional_string_parameter(consts::PARAM_PARTITION_KEY);

    let body = ctx.get_body().await;

    let db_table = app.get_table(table_name).await?;
    let sync_period = query.get_sync_period();

    let attr = http_helpers::create_transaction_attributes(app, sync_period);

    match partition_key_param {
        Some(partition_key) => {
            rows::clean_partition_and_bulk_insert(
                app,
                db_table,
                partition_key,
                body.as_slice(),
                Some(attr),
            )
            .await?;
        }
        None => {
            rows::clean_table_and_bulk_insert(app, db_table, body.as_slice(), Some(attr)).await?;
        }
    }

    return Ok(HttpOkResult::Ok);
}

pub async fn bulk_delete(
    ctx: HttpContext,
    app: &AppServices,
) -> Result<HttpOkResult, HttpFailResult> {
    let query = ctx.get_query_string();
    let table_name = query.get_query_required_string_parameter(consts::PARAM_TABLE_NAME)?;

    let body = ctx.get_body().await;

    let db_table = app.get_table(table_name).await?;
    let sync_period = query.get_sync_period();

    let attr = http_helpers::create_transaction_attributes(app, sync_period);

    rows::bulk_delete(app, db_table, body.as_slice(), Some(attr)).await;

    Ok(HttpOkResult::Ok)
}
