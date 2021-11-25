use my_http_utils::HttpFailResult;

use crate::{
    app::AppContext,
    http::{http_ctx::HttpContext, http_ok::HttpOkResult},
};

pub async fn get(app: &AppContext, ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult> {
    let query = ctx.get_query_string()?;
    let request_id: i64 = query.get_query_required_parameter("requestId")?;
    let max_records_count: usize = query.get_query_required_parameter("maxRecordsCount")?;

    let db_rows =
        crate::db_operations::read::multipart::get_next(app, request_id, max_records_count).await;

    if db_rows.is_none() {
        return Err(HttpFailResult::as_not_found("".to_string(), false));
    }

    return Ok(HttpOkResult::DbRows(db_rows.unwrap()));
}
