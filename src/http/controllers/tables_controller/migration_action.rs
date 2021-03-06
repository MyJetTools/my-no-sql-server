use flurl::FlUrl;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput, WebContentType};

use std::sync::Arc;

use crate::{
    app::AppContext,
    db_json_entity::{DbJsonEntity, JsonTimeStamp},
    db_sync::EventSource,
    http::contracts::input_params,
};

use super::models::TableMigrationInputContract;

#[my_http_server_swagger::http_route(
    method: "POST",
    route: "/Tables/MigrateFrom",
    input_data: "TableMigrationInputContract",
    description: "Migrate records from the other table of other instance",
    controller: "Tables",
    result:[
        {status_code: 200, description: "Records are migrated", model: "String"},
        {status_code: 400, description: "Table not found"},
    ]
)]
pub struct MigrationAction {
    app: Arc<AppContext>,
}

impl MigrationAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &MigrationAction,
    input_data: TableMigrationInputContract,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let db_table =
        crate::db_operations::read::table::get(action.app.as_ref(), input_data.table_name.as_str())
            .await?;

    let response = FlUrl::new(input_data.remote_url.as_str(), None)
        .append_path_segment("Row")
        .append_query_param(
            input_params::PARAM_TABLE_NAME,
            input_data.remote_table_name.as_str(),
        )
        .get()
        .await
        .unwrap();

    let body = response.receive_body().await.unwrap();

    let now = JsonTimeStamp::now();
    let rows_by_partition = DbJsonEntity::parse_as_btreemap(body.as_slice(), &now)?;

    let partitions_count = rows_by_partition.len();

    let event_src = EventSource::as_client_request(action.app.as_ref());

    crate::db_operations::write::bulk_insert_or_update::execute(
        action.app.as_ref(),
        db_table,
        rows_by_partition,
        event_src,
        &now,
        crate::db_sync::DataSynchronizationPeriod::Sec5.get_sync_moment(),
    )
    .await?;

    Ok(HttpOutput::Content {
        headers: None,
        content: format!("Migrated {} partitions", partitions_count).into_bytes(),
        content_type: Some(WebContentType::Text),
    }
    .into_ok_result(true))
}
