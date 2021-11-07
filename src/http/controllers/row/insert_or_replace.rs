use std::sync::Arc;

use my_http_utils::HttpFailResult;
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::db_json_entity::DbJsonEntity;
use crate::http::http_ctx::HttpContext;

use crate::app::AppContext;
use crate::http::http_helpers;
use crate::http::http_ok::HttpOkResult;

use super::super::consts::{self, MyNoSqlQueryString};

pub async fn post(ctx: HttpContext, app: &AppContext) -> Result<HttpOkResult, HttpFailResult> {
    let query = ctx.get_query_string()?;
    let table_name = query.get_query_required_string_parameter(consts::PARAM_TABLE_NAME)?;

    let sync_period = query.get_sync_period();

    let body = ctx.get_body().await;

    let db_table = crate::db_operations::read::table::get(app, table_name).await?;

    let attr = http_helpers::create_transaction_attributes(app, sync_period);

    let now = DateTimeAsMicroseconds::now();

    let db_json_entity = DbJsonEntity::parse(&body)?;

    let db_row = Arc::new(db_json_entity.to_db_row(now));

    let removed_db_row = crate::db_operations::write::insert_or_replace::execute(
        app,
        db_table,
        db_json_entity.partition_key,
        db_row,
        Some(attr),
    )
    .await;

    Ok(HttpOkResult::as_db_row(removed_db_row))
}
