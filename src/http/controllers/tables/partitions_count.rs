use my_http_utils::HttpFailResult;

use crate::{app::AppContext, http::http_ok::HttpOkResult};
use std::result::Result;

use super::super::consts;
use crate::http::http_ctx::HttpContext;

pub async fn get(ctx: HttpContext, app: &AppContext) -> Result<HttpOkResult, HttpFailResult> {
    let query = ctx.get_query_string()?;

    let table_name = query.get_query_required_string_parameter(consts::PARAM_TABLE_NAME)?;

    let db_table = crate::db_operations::read::table::get(app, table_name).await?;

    let partitions_amount = db_table.get_partitions_amount().await;

    return Ok(HttpOkResult::as_usize(partitions_amount));
}
