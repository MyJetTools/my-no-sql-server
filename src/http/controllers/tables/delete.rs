use my_http_utils::HttpFailResult;

use crate::{
    app::AppContext,
    http::{controllers::consts::MyNoSqlQueryString, http_helpers, http_ok::HttpOkResult},
};
use std::result::Result;

use super::super::consts;
use crate::http::http_ctx::HttpContext;

pub async fn delete(ctx: HttpContext, app: &AppContext) -> Result<HttpOkResult, HttpFailResult> {
    let query = ctx.get_query_string()?;

    let api_key = query.get_query_required_string_parameter(consts::API_KEY)?;

    if api_key != app.table_api_key.as_str() {
        return Err(HttpFailResult::as_unauthorized(None));
    }

    let table_name = query.get_query_required_string_parameter(consts::PARAM_TABLE_NAME)?;
    let sync_period = query.get_sync_period();

    let attr = http_helpers::create_transaction_attributes(app, sync_period);

    crate::db_operations::write::table::delete(app, table_name, Some(attr)).await?;

    return Ok(HttpOkResult::Ok);
}
