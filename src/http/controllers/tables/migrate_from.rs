use flurl::FlUrl;
use my_http_utils::{HttpFailResult, WebContentType};
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    app::AppContext,
    db_json_entity::DbJsonEntity,
    http::{http_ctx::HttpContext, http_helpers, http_ok::HttpOkResult},
};

use super::super::consts;

pub async fn post(ctx: HttpContext, app: &AppContext) -> Result<HttpOkResult, HttpFailResult> {
    let query = ctx.get_query_string()?;

    let remote_url = query.get_query_required_string_parameter("remoteUrl")?;
    let remote_table_name = query.get_query_required_string_parameter("remoteTableName")?;
    let table_name = query.get_query_required_string_parameter("tableName")?;

    let db_table = crate::db_operations::read::table::get(app, table_name).await?;

    let response = FlUrl::new(remote_url)
        .append_path_segment("Row")
        .append_query_param(consts::PARAM_TABLE_NAME, remote_table_name)
        .get()
        .await
        .unwrap();

    let body = response.get_body().await.unwrap();

    let now = DateTimeAsMicroseconds::now();
    let rows_by_partition = DbJsonEntity::parse_as_btreemap(body.as_slice(), now)?;

    let partitions_count = rows_by_partition.len();
    let attr = http_helpers::create_transaction_attributes(
        app,
        crate::db_sync::DataSynchronizationPeriod::Sec5,
    );

    crate::db_operations::write::bulk_insert_or_update::execute(
        app,
        db_table,
        rows_by_partition,
        Some(attr),
    )
    .await;

    Ok(HttpOkResult::Content {
        content: format!("Migrated {} partitions", partitions_count).into_bytes(),
        content_type: Some(WebContentType::Text),
    })
}
