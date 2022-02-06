use async_trait::async_trait;
use flurl::FlUrl;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, WebContentType};
use my_http_server_controllers::controllers::{
    actions::PostAction,
    documentation::{data_types::HttpDataType, out_results::HttpResult, HttpActionDescription},
};
use std::sync::Arc;

use crate::{
    app::AppContext,
    db_json_entity::{DbJsonEntity, JsonTimeStamp},
    db_sync::EventSource,
    http::contracts::input_params,
};

use super::models::TableMigrationInputContract;

pub struct MigrationAction {
    app: Arc<AppContext>,
}

impl MigrationAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}
#[async_trait]
impl PostAction for MigrationAction {
    fn get_route(&self) -> &str {
        "/Tables/MigrateFrom"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: super::consts::CONTROLLER_NAME,
            description: "Migrate records from the other table of other instance",

            input_params: TableMigrationInputContract::get_input_params().into(),
            results: vec![HttpResult {
                http_code: 200,
                nullable: true,
                description: "Records are migrated".to_string(),
                data_type: HttpDataType::as_string(),
            }],
        }
        .into()
    }

    async fn handle_request(&self, ctx: &mut HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let input_data = TableMigrationInputContract::parse_http_input(ctx).await?;

        let db_table = crate::db_operations::read::table::get(
            self.app.as_ref(),
            input_data.table_name.as_str(),
        )
        .await?;

        let response = FlUrl::new(input_data.remote_url.as_str())
            .append_path_segment("Row")
            .append_query_param(
                input_params::PARAM_TABLE_NAME,
                input_data.remote_table_name.as_str(),
            )
            .get()
            .await
            .unwrap();

        let body = response.get_body().await.unwrap();

        let now = JsonTimeStamp::now();
        let rows_by_partition = DbJsonEntity::parse_as_btreemap(body.as_slice(), &now)?;

        let partitions_count = rows_by_partition.len();

        let event_src = EventSource::as_client_request(self.app.as_ref());

        crate::db_operations::write::bulk_insert_or_update::execute(
            self.app.as_ref(),
            db_table,
            rows_by_partition,
            event_src,
            &now,
            crate::db_sync::DataSynchronizationPeriod::Sec5.get_sync_moment(),
        )
        .await;

        Ok(HttpOkResult::Content {
            headers: None,
            content: format!("Migrated {} partitions", partitions_count).into_bytes(),
            content_type: Some(WebContentType::Text),
        })
    }
}
