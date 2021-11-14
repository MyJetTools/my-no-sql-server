use my_http_utils::HttpFailResult;

use crate::{
    app::AppContext,
    http::{http_ctx::HttpContext, http_ok::HttpOkResult},
};

use super::super::consts;

pub async fn get(ctx: HttpContext, app: &AppContext) -> Result<HttpOkResult, HttpFailResult> {
    let query = ctx.get_query_string()?;

    let table_name = query.get_query_required_string_parameter(consts::PARAM_TABLE_NAME)?;

    let partition_key = query.get_query_optional_string_parameter(consts::PARAM_PARTITION_KEY);

    let db_table = crate::db_operations::read::table::get(app, table_name).await?;

    if let Some(partition_key) = partition_key {
        let table_access = db_table.data.read().await;

        let partition = table_access.get_partition(partition_key);

        if let Some(partition) = partition {
            return Ok(HttpOkResult::as_usize(partition.rows_count()));
        } else {
            return Ok(HttpOkResult::Empty);
        }
    }

    let table_access = db_table.data.read().await;

    let mut result = 0;

    for partition in table_access.get_partitions() {
        result += partition.rows_count();
    }
    return Ok(HttpOkResult::as_usize(result));
}
