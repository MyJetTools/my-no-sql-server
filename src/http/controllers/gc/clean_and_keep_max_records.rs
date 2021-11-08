use my_http_utils::HttpFailResult;

use crate::{
    app::AppContext,
    http::{http_ctx::HttpContext, http_ok::HttpOkResult},
};

use super::super::consts::{self, MyNoSqlQueryString};

pub async fn post(ctx: HttpContext, app: &AppContext) -> Result<HttpOkResult, HttpFailResult> {
    let query = ctx.get_query_string()?;
    let table_name = query.get_query_required_string_parameter(consts::PARAM_TABLE_NAME)?;

    let partition_key = query.get_query_required_string_parameter(consts::PARAM_PARTITION_KEY)?;
    //TODO- check if amount is not zero
    let amount = query.get_query_required_parameter::<usize>("amount")?;

    let sync_period = query.get_sync_period();

    let db_table = crate::db_operations::read::table::get(app, table_name).await?;

    let attr = crate::operations::transaction_attributes::create(app, sync_period);

    crate::db_operations::gc::clean_partition_and_keep_max_records::execute(
        app,
        db_table.as_ref(),
        partition_key,
        amount,
        Some(attr),
    )
    .await;

    Ok(HttpOkResult::Empty)
}
