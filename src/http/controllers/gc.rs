use crate::{
    app::AppServices,
    db::{FailOperationResult, OperationResult},
    http::{http_ctx::HttpContext, http_helpers},
};

use super::consts;

pub async fn clean_and_keep_max_partitions_amount(
    ctx: HttpContext,
    app: &AppServices,
) -> Result<OperationResult, FailOperationResult> {
    let query = ctx.get_query_string();
    let table_name = query.get_query_required_string_parameter(consts::PARAM_TABLE_NAME)?;

    let max_partitions_amount = query.get_query_required_parameter("maxAmount")?;

    let db_table = app.db.get_table(table_name).await?;
    let sync_period = query.get_sync_period();

    let attr = http_helpers::create_transaction_attributes(app, sync_period);

    crate::db_operations::gc::clean_and_keep_max_partitions_amount(
        app,
        db_table.as_ref(),
        max_partitions_amount,
        Some(attr),
    )
    .await;

    Ok(OperationResult::Ok)
}

pub async fn clean_and_keep_max_records(
    ctx: HttpContext,
    app: &AppServices,
) -> Result<OperationResult, FailOperationResult> {
    let query = ctx.get_query_string();
    let table_name = query.get_query_required_string_parameter(consts::PARAM_TABLE_NAME)?;

    let partition_key = query.get_query_required_string_parameter("partitionKey")?;
    let max_rows_amount = query.get_query_required_parameter("maxAmount")?;

    let db_table = app.db.get_table(table_name).await?;
    let sync_period = query.get_sync_period();

    let attr = http_helpers::create_transaction_attributes(app, sync_period);

    crate::db_operations::gc::clean_and_keep_max_records(
        app,
        db_table.as_ref(),
        partition_key,
        max_rows_amount,
        Some(attr),
    )
    .await;

    Ok(OperationResult::Ok)
}
