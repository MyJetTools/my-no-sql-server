use crate::{
    app::AppServices,
    db::{FailOperationResult, OperationResult},
    db_operations::{rows, tables},
    http::http_helpers,
};
use std::result::Result;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::consts;
use crate::http::http_ctx::HttpContext;

#[derive(Deserialize, Serialize)]
pub struct TableJsonResult {
    pub name: String,
}

pub async fn list_of_tables(app: Arc<AppServices>) -> Result<OperationResult, FailOperationResult> {
    let tables = app.db.get_table_names().await;

    let mut response: Vec<TableJsonResult> = vec![];

    for name in tables {
        response.push(TableJsonResult { name });
    }

    let json = serde_json::to_string(&response).unwrap();

    return Ok(OperationResult::OkWithJsonString { json });
}

pub async fn create_table(
    ctx: HttpContext,
    app: Arc<AppServices>,
) -> Result<OperationResult, FailOperationResult> {
    let query = ctx.get_query_string();

    let table_name = query.get_query_required_string_parameter(consts::PARAM_TABLE_NAME)?;

    let persist_table = query.get_query_bool_parameter(consts::PARAM_PERSIST_TABLE, true);

    let max_partitions_amount =
        query.get_query_optional_parameter(consts::PARAM_MAX_PARTITIONS_AMOUNT);

    let sync_period = query.get_sync_period();

    let attr = http_helpers::create_transaction_attributes(app.as_ref(), sync_period);

    tables::create_table(
        app.as_ref(),
        table_name,
        persist_table,
        max_partitions_amount,
        Some(attr),
    )
    .await?;

    return Ok(OperationResult::Ok);
}

pub async fn create_table_if_not_exists(
    ctx: HttpContext,
    app: Arc<AppServices>,
) -> Result<OperationResult, FailOperationResult> {
    let query = ctx.get_query_string();

    let table_name = query.get_query_required_string_parameter(consts::PARAM_TABLE_NAME)?;
    let persist_table = query.get_query_bool_parameter(consts::PARAM_PERSIST_TABLE, true);

    let max_partitions_amount =
        query.get_query_optional_parameter(consts::PARAM_MAX_PARTITIONS_AMOUNT);

    let sync_period = query.get_sync_period();

    let attr = http_helpers::create_transaction_attributes(app.as_ref(), sync_period);

    tables::create_table_if_not_exist(
        app.as_ref(),
        table_name,
        persist_table,
        max_partitions_amount,
        Some(attr),
    )
    .await;

    return Ok(OperationResult::Ok);
}

pub async fn clean(
    ctx: HttpContext,
    app: Arc<AppServices>,
) -> Result<OperationResult, FailOperationResult> {
    let query = ctx.get_query_string();

    let table_name = query.get_query_required_string_parameter(consts::PARAM_TABLE_NAME)?;
    let sync_period = query.get_sync_period();

    let db_table = app.db.get_table(table_name).await?;

    let attr = http_helpers::create_transaction_attributes(app.as_ref(), sync_period);

    rows::clean_table(app.as_ref(), db_table.as_ref(), Some(attr)).await;

    return Ok(OperationResult::Ok);
}

pub async fn update_persist(
    ctx: HttpContext,
    app: Arc<AppServices>,
) -> Result<OperationResult, FailOperationResult> {
    let query = ctx.get_query_string();

    let table_name = query.get_query_required_string_parameter(consts::PARAM_TABLE_NAME)?;
    let sync_period = query.get_sync_period();

    let persist = query.get_query_bool_parameter("persist", true);

    let max_partitions_amount = query.get_query_optional_parameter("maxPartitionsAmount");

    let db_table = app.db.get_table(table_name).await?;

    let attr = http_helpers::create_transaction_attributes(app.as_ref(), sync_period);

    tables::set_table_attrubutes(
        app.as_ref(),
        db_table.as_ref(),
        persist,
        max_partitions_amount,
        Some(attr),
    )
    .await;

    return Ok(OperationResult::Ok);
}

pub async fn get_partitions_count(
    ctx: HttpContext,
    app: Arc<AppServices>,
) -> Result<OperationResult, FailOperationResult> {
    let query = ctx.get_query_string();

    let table_name = query.get_query_required_string_parameter(consts::PARAM_TABLE_NAME)?;

    let db_table = app.db.get_table(table_name).await?;

    let partitions_count = db_table.get_partitions_amount().await;

    return Ok(OperationResult::Number {
        value: partitions_count as i64,
    });
}