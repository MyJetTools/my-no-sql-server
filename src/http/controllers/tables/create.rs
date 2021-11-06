use my_http_utils::HttpFailResult;

use crate::{
    app::AppContext,
    http::{
        controllers::consts::MyNoSqlQueryString, http_ctx::HttpContext, http_helpers,
        http_ok::HttpOkResult,
    },
};

use super::super::consts;
pub async fn post(ctx: HttpContext, app: &AppContext) -> Result<HttpOkResult, HttpFailResult> {
    let query = ctx.get_query_string()?;

    let table_name = query.get_query_required_string_parameter(consts::PARAM_TABLE_NAME)?;

    let persist_table = query.get_query_bool_parameter(consts::PARAM_PERSIST_TABLE, true);

    let max_partitions_amount =
        query.get_query_optional_parameter(consts::PARAM_MAX_PARTITIONS_AMOUNT);

    let sync_period = query.get_sync_period();

    let attr = http_helpers::create_transaction_attributes(app, sync_period);

    crate::db_operations::write::table::create(
        app,
        table_name,
        persist_table,
        max_partitions_amount,
        Some(attr),
    )
    .await?;

    return Ok(HttpOkResult::Ok);
}
