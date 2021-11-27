use my_http_utils::HttpFailResult;

use crate::{
    app::AppContext,
    http::{
        controllers::consts::MyNoSqlQueryString, http_ok::HttpOkResult,
        params_readers::StandardParamsReader,
    },
};
use std::result::Result;

use super::super::consts;
use crate::http::http_ctx::HttpContext;

pub async fn delete(ctx: HttpContext, app: &AppContext) -> Result<HttpOkResult, HttpFailResult> {
    let query = ctx.get_query_string()?;

    let api_key = ctx.get_api_key()?;

    if api_key != app.table_api_key.as_str() {
        return Err(HttpFailResult::as_unauthorized(None));
    }

    let table_name = query.get_query_required_string_parameter(consts::PARAM_TABLE_NAME)?;
    let sync_period = query.get_sync_period();

    let attr = crate::operations::transaction_attributes::create(app, sync_period);

    crate::db_operations::write::table::delete(app, table_name, Some(attr)).await?;

    return Ok(HttpOkResult::Empty);
}
