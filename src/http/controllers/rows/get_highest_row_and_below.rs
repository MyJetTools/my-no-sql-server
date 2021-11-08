use my_http_utils::HttpFailResult;

use crate::{
    app::AppContext,
    http::{http_ctx::HttpContext, http_ok::HttpOkResult},
};

use super::super::consts;

pub async fn get(ctx: HttpContext, app: &AppContext) -> Result<HttpOkResult, HttpFailResult> {
    let query = ctx.get_query_string()?;
    let table_name = query.get_query_required_string_parameter(consts::PARAM_TABLE_NAME)?;

    let partition_key = query.get_query_required_string_parameter(consts::PARAM_PARTITION_KEY)?;
    let row_key = query.get_query_required_string_parameter(consts::PARAM_ROW_KEY)?;

    let max_amount = query.get_query_required_parameter("maxAmount")?;

    let db_table = crate::db_operations::read::table::get(app, table_name).await?;

    let result = crate::db_operations::read::get_highest_row_and_below::execute(
        db_table.as_ref(),
        partition_key,
        row_key,
        max_amount,
    )
    .await;

    Ok(result.into())
}
