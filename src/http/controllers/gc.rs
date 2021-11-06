use my_http_utils::HttpFailResult;

use crate::{
    app::AppContext,
    http::{http_ctx::HttpContext, http_helpers, http_ok::HttpOkResult},
};

use super::consts::{self, MyNoSqlQueryString};

pub async fn clean_and_keep_max_partitions_amount(
    ctx: HttpContext,
    app: &AppContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let query = ctx.get_query_string()?;
    let table_name = query.get_query_required_string_parameter(consts::PARAM_TABLE_NAME)?;

    let max_partitions_amount = query.get_query_required_parameter("maxAmount")?;

    let db_table = crate::db_operations::read::table::get(app, table_name).await?;

    let sync_period = query.get_sync_period();

    let attr = http_helpers::create_transaction_attributes(app, sync_period);

    crate::db_operations::gc::keep_max_partitions_amount::execute(
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
    app: &AppContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let query = ctx.get_query_string()?;
    let table_name = query.get_query_required_string_parameter(consts::PARAM_TABLE_NAME)?;

    let partition_key = query.get_query_required_string_parameter("partitionKey")?;
    let max_rows_amount = query.get_query_required_parameter("maxAmount")?;

    let db_table = crate::db_operations::read::table::get(app, table_name).await?;

    let sync_period = query.get_sync_period();

    let attr = http_helpers::create_transaction_attributes(app, sync_period);

    crate::db_operations::gc::clean_partition_and_keep_max_records::execute(
        app,
        db_table,
        partition_key,
        max_rows_amount,
        Some(attr),
    )
    .await;

    Ok(HttpOkResult::Ok)
}
