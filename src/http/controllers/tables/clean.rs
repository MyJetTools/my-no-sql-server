use my_http_utils::HttpFailResult;

use crate::{
    app::AppContext,
    http::{controllers::consts::MyNoSqlQueryString, http_ok::HttpOkResult},
};
use std::result::Result;

use super::super::consts;
use crate::http::http_ctx::HttpContext;

pub async fn put(ctx: HttpContext, app: &AppContext) -> Result<HttpOkResult, HttpFailResult> {
    let query = ctx.get_query_string()?;

    let table_name = query.get_query_required_string_parameter(consts::PARAM_TABLE_NAME)?;
    let sync_period = query.get_sync_period();

    let db_table = crate::db_operations::read::table::get(app, table_name).await?;

    let attr = crate::operations::transaction_attributes::create(app, sync_period);

    crate::db_operations::write::clean_table::execute(app, db_table, Some(attr)).await;

    return Ok(HttpOkResult::Empty);
}
