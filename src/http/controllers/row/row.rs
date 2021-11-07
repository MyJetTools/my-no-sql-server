use my_http_utils::HttpFailResult;

use crate::db_json_entity::JsonTimeStamp;
use crate::http::controllers::consts::MyNoSqlQueryString;
use crate::http::http_helpers;
use crate::{db_operations, http::http_ctx::HttpContext};

use crate::app::AppContext;
use crate::http::http_ok::HttpOkResult;

use super::super::consts::{self};

pub async fn get(ctx: HttpContext, app: &AppContext) -> Result<HttpOkResult, HttpFailResult> {
    let query = ctx.get_query_string()?;
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

pub async fn delete(ctx: HttpContext, app: &AppContext) -> Result<HttpOkResult, HttpFailResult> {
    let query = ctx.get_query_string()?;
    let table_name = query.get_query_required_string_parameter(consts::PARAM_TABLE_NAME)?;

    let partition_key = query.get_query_required_string_parameter(consts::PARAM_PARTITION_KEY)?;
    let row_key = query.get_query_required_string_parameter(consts::PARAM_ROW_KEY)?;

    let sync_period = query.get_sync_period();

    let db_table = crate::db_operations::read::table::get(app, table_name).await?;

    let attr = http_helpers::create_transaction_attributes(app, sync_period);
    let now = JsonTimeStamp::now();
    let result = db_operations::write::delete_row::execute(
        app,
        db_table,
        partition_key,
        row_key,
        Some(attr),
        &now,
    )
    .await;

    Ok(HttpOkResult::as_db_row(result))
}
