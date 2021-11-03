use std::sync::Arc;

use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::db_json_entity::DbJsonEntity;
use crate::http::http_ctx::HttpContext;

use crate::app::AppContext;
use crate::http::http_fail::HttpFailResult;
use crate::http::http_helpers;
use crate::http::http_ok::HttpOkResult;

use super::consts;

pub async fn get_rows(ctx: HttpContext, app: &AppContext) -> Result<HttpOkResult, HttpFailResult> {
    let query = ctx.get_query_string();
    let table_name = query.get_query_required_string_parameter(consts::PARAM_TABLE_NAME)?;

    let partition_key = query.get_query_optional_string_parameter(consts::PARAM_PARTITION_KEY);
    let row_key = query.get_query_optional_string_parameter(consts::PARAM_ROW_KEY);

    let db_table = crate::db_operations::read::table::get(app, table_name).await?;

    if let Some(partition_key) = partition_key {
        if let Some(row_key) = row_key {
            let result = crate::db_operations::read::rows::get_row(
                db_table.as_ref(),
                partition_key,
                row_key,
            )
            .await;

            return Ok(result.into());
        } else {
            let limit = query.get_query_optional_parameter::<usize>(consts::PARAM_LIMIT);
            let skip = query.get_query_optional_parameter::<usize>(consts::PARAM_SKIP);

            let result = crate::db_operations::read::rows::get_all_rows_by_partition_key(
                db_table.as_ref(),
                partition_key,
                limit,
                skip,
            )
            .await;

            return Ok(result.into());
        }
    } else {
        if let Some(row_key) = row_key {
            let limit = query.get_query_optional_parameter::<usize>(consts::PARAM_LIMIT);
            let skip = query.get_query_optional_parameter::<usize>(consts::PARAM_SKIP);

            let result = crate::db_operations::read::rows::get_all_rows_by_row_key(
                db_table.as_ref(),
                row_key,
                limit,
                skip,
            )
            .await;

            return Ok(result.into());
        } else {
            let limit = query.get_query_optional_parameter::<usize>(consts::PARAM_LIMIT);
            let skip = query.get_query_optional_parameter::<usize>(consts::PARAM_SKIP);

            let result = crate::db_operations::read::rows::get_all_table_rows(
                db_table.as_ref(),
                limit,
                skip,
            )
            .await;

            return Ok(result.into());
        }
    }
}

pub async fn insert(ctx: HttpContext, app: &AppContext) -> Result<HttpOkResult, HttpFailResult> {
    let query = ctx.get_query_string();
    let table_name = query.get_query_required_string_parameter(consts::PARAM_TABLE_NAME)?;

    let sync_period = query.get_sync_period();

    let body = ctx.get_body().await;

    let db_table = crate::db_operations::read::table::get(app, table_name).await?;
    let now = DateTimeAsMicroseconds::now();
    let db_json_entity = DbJsonEntity::parse(&body)?;

    crate::db_operations::write::insert::validate_before(
        db_table.as_ref(),
        db_json_entity.partition_key,
        db_json_entity.row_key,
    )
    .await?;

    let attr = http_helpers::create_transaction_attributes(app, sync_period);

    let db_row = Arc::new(db_json_entity.to_db_row(now));

    crate::db_operations::write::insert::execute(
        app,
        db_table.as_ref(),
        db_json_entity.partition_key,
        db_row,
        Some(attr),
    )
    .await?;

    Ok(HttpOkResult::Ok)
}

pub async fn insert_or_replace(
    ctx: HttpContext,
    app: &AppContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let query = ctx.get_query_string();
    let table_name = query.get_query_required_string_parameter(consts::PARAM_TABLE_NAME)?;

    let sync_period = query.get_sync_period();

    let body = ctx.get_body().await;

    let db_table = crate::db_operations::read::table::get(app, table_name).await?;

    let attr = http_helpers::create_transaction_attributes(app, sync_period);

    let now = DateTimeAsMicroseconds::now();

    let db_json_entity = DbJsonEntity::parse(&body)?;

    let db_row = Arc::new(db_json_entity.to_db_row(now));

    crate::db_operations::write::insert_or_replace::execute(
        app,
        db_table,
        db_json_entity.partition_key,
        db_row,
        Some(attr),
    )
    .await;

    Ok(HttpOkResult::Ok)
}

pub async fn replace(ctx: HttpContext, app: &AppContext) -> Result<HttpOkResult, HttpFailResult> {
    let query = ctx.get_query_string();
    let table_name = query.get_query_required_string_parameter(consts::PARAM_TABLE_NAME)?;

    let sync_period = query.get_sync_period();

    let body = ctx.get_body().await;

    let db_table = crate::db_operations::read::table::get(app, table_name).await?;

    let now = DateTimeAsMicroseconds::now();

    let db_json_entity = DbJsonEntity::parse(&body)?;

    crate::db_operations::write::replace::validate_before(
        db_table.as_ref(),
        db_json_entity.partition_key,
        db_json_entity.row_key,
        db_json_entity.time_stamp,
    )
    .await?;

    let db_row = Arc::new(db_json_entity.to_db_row(now));

    let attr = http_helpers::create_transaction_attributes(app, sync_period);

    crate::db_operations::write::replace::execute(
        app,
        db_table.as_ref(),
        db_json_entity.partition_key,
        db_row,
        Some(attr),
        db_json_entity.time_stamp.unwrap(),
    )
    .await?;

    Ok(HttpOkResult::Ok)
}
