use my_http_utils::HttpFailResult;

use crate::db_json_entity::{DbJsonEntity, JsonTimeStamp};
use crate::http::http_ctx::HttpContext;

use crate::app::AppContext;

use crate::http::http_ok::HttpOkResult;

use super::super::consts::{self, MyNoSqlQueryString};

pub async fn post(ctx: HttpContext, app: &AppContext) -> Result<HttpOkResult, HttpFailResult> {
    let query = ctx.get_query_string()?;
    let table_name = query.get_query_required_string_parameter(consts::PARAM_TABLE_NAME)?;

    let partition_key_param =
        query.get_query_optional_string_parameter(consts::PARAM_PARTITION_KEY);

    let body = ctx.get_body().await;

    let db_table = crate::db_operations::read::table::get(app, table_name).await?;
    let sync_period = query.get_sync_period();

    let attr = crate::operations::transaction_attributes::create(app, sync_period);
    let now = JsonTimeStamp::now();

    let rows_by_partition = DbJsonEntity::parse_as_btreemap(body.as_slice(), &now)?;

    match partition_key_param {
        Some(partition_key) => {
            crate::db_operations::write::clean_partition_and_bulk_insert::execute(
                app,
                db_table,
                partition_key,
                rows_by_partition,
                Some(attr),
                &now,
            )
            .await?;
        }
        None => {
            crate::db_operations::write::clean_table_and_bulk_insert::execute(
                app,
                db_table,
                rows_by_partition,
                Some(attr),
                &now,
            )
            .await?;
        }
    }

    return Ok(HttpOkResult::Empty);
}
