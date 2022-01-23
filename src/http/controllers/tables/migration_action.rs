use async_trait::async_trait;
use flurl::FlUrl;
use my_http_server::{
    middlewares::controllers::{
        actions::PostAction,
        documentation::{
            data_types::{HttpDataType, HttpField, HttpObjectStructure},
            in_parameters::{HttpInputParameter, HttpParameterInputSource},
            out_results::HttpResult,
            HttpActionDescription,
        },
    },
    HttpContext, HttpFailResult, HttpOkResult, WebContentType,
};
use std::sync::Arc;

use crate::{
    app::AppContext,
    db_json_entity::{DbJsonEntity, JsonTimeStamp},
    http::contracts::input_params,
};

const PARAM_REMOTE_URL: &str = "remoteUrl";
const PARAM_REMOTE_TABLE_NAME: &str = "remoteTableName";

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

            input_params: vec![
                HttpInputParameter {
                    field: HttpField::new(PARAM_REMOTE_URL, HttpDataType::as_string(), true, None),
                    description: "Url of the remote MyNoSqlServer we are going to copy data from"
                        .to_string(),
                    source: HttpParameterInputSource::Query,
                },
                HttpInputParameter {
                    field: HttpField::new(
                        PARAM_REMOTE_TABLE_NAME,
                        HttpDataType::as_string(),
                        true,
                        None,
                    ),
                    description:
                        "Table name of the remote MyNoSqlServer we are going to copy data from"
                            .to_string(),
                    source: HttpParameterInputSource::Query,
                },
                HttpInputParameter {
                    field: HttpField::new(
                        input_params::PARAM_TABLE_NAME,
                        HttpDataType::as_string(),
                        true,
                        None,
                    ),
                    description:
                        "Table name of the current MyNoSqlServer we are going to copy data to"
                            .to_string(),
                    source: HttpParameterInputSource::Query,
                },
            ]
            .into(),
            results: vec![HttpResult {
                http_code: 200,
                nullable: true,
                description: "Records are migrated".to_string(),
                data_type: HttpDataType::as_string(),
            }],
        }
        .into()
    }

    async fn handle_request(&self, ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let query = ctx.get_query_string()?;

        let remote_url = query.get_required_string_parameter(PARAM_REMOTE_URL)?;
        let remote_table_name = query.get_required_string_parameter(PARAM_REMOTE_TABLE_NAME)?;
        let table_name = query.get_required_string_parameter(input_params::PARAM_TABLE_NAME)?;

        let db_table =
            crate::db_operations::read::table::get(self.app.as_ref(), table_name).await?;

        let response = FlUrl::new(remote_url)
            .append_path_segment("Row")
            .append_query_param(input_params::PARAM_TABLE_NAME, remote_table_name)
            .get()
            .await
            .unwrap();

        let body = response.get_body().await.unwrap();

        let now = JsonTimeStamp::now();
        let rows_by_partition = DbJsonEntity::parse_as_btreemap(body.as_slice(), &now)?;

        let partitions_count = rows_by_partition.len();
        let attr = crate::operations::transaction_attributes::create(
            self.app.as_ref(),
            crate::db_sync::DataSynchronizationPeriod::Sec5,
        );

        crate::db_operations::write::bulk_insert_or_update::execute(
            self.app.as_ref(),
            db_table,
            rows_by_partition,
            Some(attr),
            &now,
        )
        .await;

        Ok(HttpOkResult::Content {
            content: format!("Migrated {} partitions", partitions_count).into_bytes(),
            content_type: Some(WebContentType::Text),
        })
    }
}
