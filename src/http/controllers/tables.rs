use crate::{
    app::AppContext,
    http::{http_fail::HttpFailResult, http_helpers, http_ok::HttpOkResult},
};
use std::result::Result;

use serde::{Deserialize, Serialize};

use super::consts;
use crate::http::http_ctx::HttpContext;

#[derive(Deserialize, Serialize)]
pub struct TableJsonResult {
    pub name: String,
    pub persist: bool,
    #[serde(rename = "maxPartitionsAmount")]
    pub max_partitions_amount: Option<usize>,
}

pub async fn list_of_tables(app: &AppContext) -> Result<HttpOkResult, HttpFailResult> {
    let tables = app.db.get_tables().await;

    let mut response: Vec<TableJsonResult> = vec![];

    for db_table in &tables {
        let attr = db_table.get_attributes().await;
        response.push(TableJsonResult {
            name: db_table.name.to_string(),
            persist: attr.persist,
            max_partitions_amount: attr.max_partitions_amount,
        });
    }

    return HttpOkResult::create_json_response(response);
}

pub async fn create_table(
    ctx: HttpContext,
    app: &AppContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let query = ctx.get_query_string();

    let table_name = query.get_query_required_string_parameter(consts::PARAM_TABLE_NAME)?;

    let persist_table = query.get_query_bool_parameter(consts::PARAM_PERSIST_TABLE, true);

    let max_partitions_amount =
        query.get_query_optional_parameter(consts::PARAM_MAX_PARTITIONS_AMOUNT);

    let sync_period = query.get_sync_period();

    let attr = http_helpers::create_transaction_attributes(app, sync_period);

    crate::db_operations::write::table::create(
        app,
        table_name,
        persist_table,
        max_partitions_amount,
        Some(attr),
    )
    .await?;

    return Ok(HttpOkResult::Ok);
}

pub async fn create_table_if_not_exists(
    ctx: HttpContext,
    app: &AppContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let query = ctx.get_query_string();

    let table_name = query.get_query_required_string_parameter(consts::PARAM_TABLE_NAME)?;
    let persist_table = query.get_query_bool_parameter(consts::PARAM_PERSIST_TABLE, true);

    let max_partitions_amount =
        query.get_query_optional_parameter(consts::PARAM_MAX_PARTITIONS_AMOUNT);

    let sync_period = query.get_sync_period();

    let attr = http_helpers::create_transaction_attributes(app, sync_period);

    crate::db_operations::write::table::create_if_not_exist(
        app,
        table_name,
        persist_table,
        max_partitions_amount,
        Some(attr),
    )
    .await;

    return Ok(HttpOkResult::Ok);
}

pub async fn clean(ctx: HttpContext, app: &AppContext) -> Result<HttpOkResult, HttpFailResult> {
    let query = ctx.get_query_string();

    let table_name = query.get_query_required_string_parameter(consts::PARAM_TABLE_NAME)?;
    let sync_period = query.get_sync_period();

    let db_table = crate::db_operations::read::table::get(app, table_name).await?;

    let attr = http_helpers::create_transaction_attributes(app, sync_period);

    crate::db_operations::write::clean_table::execute(app, db_table, Some(attr)).await;

    return Ok(HttpOkResult::Ok);
}

pub async fn update_persist(
    ctx: HttpContext,
    app: &AppContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let query = ctx.get_query_string();

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

pub async fn get_partitions_count(
    ctx: HttpContext,
    app: &AppContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let query = ctx.get_query_string();

    let table_name = query.get_query_required_string_parameter(consts::PARAM_TABLE_NAME)?;

    let db_table = crate::db_operations::read::table::get(app, table_name).await?;

    let partitions_count = db_table.get_partitions_amount().await;

    return HttpOkResult::create_as_usize(partitions_count);
}

pub async fn delete(ctx: HttpContext, app: &AppContext) -> Result<HttpOkResult, HttpFailResult> {
    let query = ctx.get_query_string();

    let api_key = query.get_query_required_string_parameter(consts::API_KEY)?;

    if api_key != app.table_api_key.as_str() {
        return Err(HttpFailResult::as_unauthorized());
    }

    let table_name = query.get_query_required_string_parameter(consts::PARAM_TABLE_NAME)?;
    let sync_period = query.get_sync_period();

    let attr = http_helpers::create_transaction_attributes(app, sync_period);

    crate::db_operations::write::table::delete(app, table_name, Some(attr)).await?;

    return Ok(HttpOkResult::Ok);
}
