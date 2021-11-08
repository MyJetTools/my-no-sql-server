use std::collections::HashMap;

use my_http_utils::HttpFailResult;

use crate::db_json_entity::JsonTimeStamp;
use crate::http::http_ctx::HttpContext;

use crate::app::AppContext;
use crate::http::http_helpers;
use crate::http::http_ok::HttpOkResult;

use super::super::consts::{self, MyNoSqlQueryString};

pub async fn post(ctx: HttpContext, app: &AppContext) -> Result<HttpOkResult, HttpFailResult> {
    let query = ctx.get_query_string()?;
    let table_name = query.get_query_required_string_parameter(consts::PARAM_TABLE_NAME)?;

    let body = ctx.get_body().await;

    let db_table = crate::db_operations::read::table::get(app, table_name).await?;
    let sync_period = query.get_sync_period();

    let attr = http_helpers::create_transaction_attributes(app, sync_period);

    let rows_to_delete: HashMap<String, Vec<String>> =
        serde_json::from_slice(body.as_slice()).unwrap();

    let now = JsonTimeStamp::now();

    crate::db_operations::write::bulk_delete::execute(
        app,
        db_table,
        rows_to_delete,
        Some(attr),
        &now,
    )
    .await;

    Ok(HttpOkResult::Empty)
}
