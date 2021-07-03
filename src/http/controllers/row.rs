use std::sync::Arc;

use crate::db_operations::rows;
use crate::http::http_ctx::HttpContext;

use crate::http::http_helpers;
use crate::{
    app::AppServices,
    db::{FailOperationResult, OperationResult},
};

use super::consts;

pub async fn get_rows(
    ctx: HttpContext,
    app: Arc<AppServices>,
) -> Result<OperationResult, FailOperationResult> {
    let query = ctx.get_query_string();
    let table_name = query.get_query_required_string_parameter(consts::PARAM_TABLE_NAME)?;

    let partition_key = query.get_query_optional_string_parameter(consts::PARAM_PARTITION_KEY);
    let row_key = query.get_query_optional_string_parameter(consts::PARAM_ROW_KEY);

    let db_table = app.db.get_table(table_name).await?;

    return db_table.get_rows(partition_key, row_key).await;
}

pub async fn insert(
    ctx: HttpContext,
    app: Arc<AppServices>,
) -> Result<OperationResult, FailOperationResult> {
    let query = ctx.get_query_string();
    let table_name = query.get_query_required_string_parameter(consts::PARAM_TABLE_NAME)?;

    let sync_period = query.get_sync_period();

    let body = ctx.get_body().await;

    let db_table = app.db.get_table(table_name).await?;

    let attr = http_helpers::create_transaction_attributes(app.as_ref(), sync_period);

    return rows::insert(app.as_ref(), db_table.as_ref(), &body, Some(attr)).await;
}

pub async fn insert_or_replace(
    ctx: HttpContext,
    app: Arc<AppServices>,
) -> Result<OperationResult, FailOperationResult> {
    let query = ctx.get_query_string();
    let table_name = query.get_query_required_string_parameter(consts::PARAM_TABLE_NAME)?;

    let sync_period = query.get_sync_period();

    let body = ctx.get_body().await;

    let db_table = app.db.get_table(table_name).await?;

    let attr = http_helpers::create_transaction_attributes(app.as_ref(), sync_period);

    return rows::insert_or_replace(app.as_ref(), db_table.as_ref(), &body, Some(attr)).await;
}

pub async fn replace(
    ctx: HttpContext,
    app: Arc<AppServices>,
) -> Result<OperationResult, FailOperationResult> {
    let query = ctx.get_query_string();
    let table_name = query.get_query_required_string_parameter(consts::PARAM_TABLE_NAME)?;

    let sync_period = query.get_sync_period();

    let body = ctx.get_body().await;

    let db_table = app.db.get_table(table_name).await?;
    let attr = http_helpers::create_transaction_attributes(app.as_ref(), sync_period);

    return rows::replace(app.as_ref(), db_table.as_ref(), body.as_slice(), Some(attr)).await;
}
