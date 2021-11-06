use my_http_utils::HttpFailResult;

use crate::{
    app::AppContext,
    http::{controllers::consts::MyNoSqlQueryString, http_helpers, http_ok::HttpOkResult},
};
use std::result::Result;

use super::super::consts;
use crate::http::http_ctx::HttpContext;

pub async fn post(ctx: HttpContext, app: &AppContext) -> Result<HttpOkResult, HttpFailResult> {
    let query = ctx.get_query_string()?;

    let table_name = query.get_query_required_string_parameter(consts::PARAM_TABLE_NAME)?;
    let sync_period = query.get_sync_period();

    let persist = query.get_query_bool_parameter("persist", true);

    let max_partitions_amount = query.get_query_optional_parameter("maxPartitionsAmount");

    let db_table = crate::db_operations::read::table::get(app, table_name).await?;

    let attr = http_helpers::create_transaction_attributes(app, sync_period);

    crate::db_operations::write::table::set_table_attrubutes(
        app,
        db_table,
        false,
        persist,
        max_partitions_amount,
        Some(attr),
    )
    .await;

    return Ok(HttpOkResult::Ok);
}
