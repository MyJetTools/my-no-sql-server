use crate::{
    app::AppServices,
    http::{http_ctx::HttpContext, http_fail::HttpFailResult, http_helpers, http_ok::HttpOkResult},
};

use super::consts;

pub async fn clean_and_keep_max_partitions_amount(
    ctx: HttpContext,
    app: &AppServices,
) -> Result<HttpOkResult, HttpFailResult> {
    let query = ctx.get_query_string();
    let table_name = query.get_query_required_string_parameter(consts::PARAM_TABLE_NAME)?;

    let max_partitions_amount = query.get_query_required_parameter("maxAmount")?;

    let db_table = app.get_table(table_name).await?;
    let sync_period = query.get_sync_period();

    let attr = http_helpers::create_transaction_attributes(app, sync_period);

    crate::db_operations::gc::keep_max_partitions_amount(
        app,
        db_table,
        max_partitions_amount,
        Some(attr),
    )
    .await;

    Ok(HttpOkResult::Ok)
}

pub async fn clean_and_keep_max_records(
    ctx: HttpContext,
    app: &AppServices,
) -> Result<HttpOkResult, HttpFailResult> {
    let query = ctx.get_query_string();
    let table_name = query.get_query_required_string_parameter(consts::PARAM_TABLE_NAME)?;

    let partition_key = query.get_query_required_string_parameter("partitionKey")?;
    let max_rows_amount = query.get_query_required_parameter("maxAmount")?;

    let db_table = app.get_table(table_name).await?;
    let sync_period = query.get_sync_period();

    let attr = http_helpers::create_transaction_attributes(app, sync_period);

    crate::db_operations::gc::clean_and_keep_max_records(
        app,
        db_table,
        partition_key,
        max_rows_amount,
        Some(attr),
    )
    .await;

    Ok(HttpOkResult::Ok)
}

pub async fn execute(ctx: HttpContext, app: &AppServices) -> Result<HttpOkResult, HttpFailResult> {
    let query = ctx.get_query_string();
    let table_name = query.get_query_required_string_parameter(consts::PARAM_TABLE_NAME)?;

    let db_table = app.get_table(table_name).await?;

    let db_table_attributes = db_table.get_attributes().await;

    if db_table_attributes.max_partitions_amount.is_none() {
        return Ok(HttpOkResult::Ok);
    }

    let max_partitions_amount = db_table_attributes.max_partitions_amount.unwrap();

    let sync_period = query.get_sync_period();

    let attr = http_helpers::create_transaction_attributes(app, sync_period);

    crate::db_operations::gc::keep_max_partitions_amount(
        app,
        db_table,
        max_partitions_amount,
        Some(attr),
    )
    .await;

    Ok(HttpOkResult::Ok)
}
