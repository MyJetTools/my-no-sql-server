use std::sync::Arc;

use crate::{
    app::AppServices,
    db::{FailOperationResult, OperationResult},
    http::http_ctx::HttpContext,
};

use super::consts;

pub async fn get_single_partition_multiple_rows(
    ctx: HttpContext,
    app: Arc<AppServices>,
) -> Result<OperationResult, FailOperationResult> {
    let query = ctx.get_query_string();
    let table_name = query.get_query_required_string_parameter(consts::PARAM_TABLE_NAME)?;

    let partition_key = query.get_query_required_string_parameter(consts::PARAM_PARTITION_KEY)?;

    let body = ctx.get_body().await;

    let db_table = app.db.get_table(table_name).await?;

    let row_keys = serde_json::from_slice(body.as_slice()).unwrap();

    return db_table
        .get_single_partition_multiple_rows(partition_key, row_keys)
        .await;
}

pub async fn get_highest_row_and_below(
    ctx: HttpContext,
    app: Arc<AppServices>,
) -> Result<OperationResult, FailOperationResult> {
    let query = ctx.get_query_string();
    let table_name = query.get_query_required_string_parameter(consts::PARAM_TABLE_NAME)?;

    let partition_key = query.get_query_required_string_parameter(consts::PARAM_PARTITION_KEY)?;
    let row_key = query.get_query_required_string_parameter(consts::PARAM_ROW_KEY)?;

    let max_amount = query.get_query_required_parameter("maxAmount")?;

    let db_table = app.db.get_table(table_name).await?;

    return db_table
        .get_highest_row_and_below(partition_key, row_key.to_string(), max_amount)
        .await;
}
