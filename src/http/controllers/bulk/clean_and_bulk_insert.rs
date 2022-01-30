use std::sync::Arc;

use my_http_server::middlewares::controllers::actions::PostAction;
use my_http_server::middlewares::controllers::documentation::data_types::HttpDataType;
use my_http_server::middlewares::controllers::documentation::out_results::HttpResult;
use my_http_server::middlewares::controllers::documentation::HttpActionDescription;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult};

use crate::db_json_entity::{DbJsonEntity, JsonTimeStamp};

use crate::app::AppContext;
use crate::db_sync::EventSource;

use super::models::CleanAndBulkInsertInputContract;

pub struct CleanAndBulkInsertControllerAction {
    app: Arc<AppContext>,
}

impl CleanAndBulkInsertControllerAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl PostAction for CleanAndBulkInsertControllerAction {
    fn get_route(&self) -> &str {
        "/Bulk/CleanAndBulkInsert"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: super::consts::CONTROLLER_NAME,
            description: "Cleans partition and does bulk insert operation transactionally",

            input_params: CleanAndBulkInsertInputContract::get_input_params().into(),
            results: vec![HttpResult {
                http_code: 202,
                nullable: true,
                description: "Successful operation".to_string(),
                data_type: HttpDataType::None,
            }],
        }
        .into()
    }

    async fn handle_request(&self, ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let input_data = CleanAndBulkInsertInputContract::parse_http_input(ctx).await?;

        let db_table = crate::db_operations::read::table::get(
            self.app.as_ref(),
            input_data.table_name.as_str(),
        )
        .await?;

        let event_src = EventSource::as_client_request(self.app.as_ref(), input_data.sync_period);

        let now = JsonTimeStamp::now();

        let rows_by_partition = DbJsonEntity::parse_as_btreemap(input_data.body.as_slice(), &now)?;

        match &input_data.partition_key {
            Some(partition_key) => {
                crate::db_operations::write::clean_partition_and_bulk_insert::execute(
                    self.app.as_ref(),
                    db_table,
                    partition_key,
                    rows_by_partition,
                    Some(event_src),
                    &now,
                )
                .await?;
            }
            None => {
                crate::db_operations::write::clean_table_and_bulk_insert::execute(
                    self.app.as_ref(),
                    db_table,
                    rows_by_partition,
                    Some(event_src),
                    &now,
                )
                .await?;
            }
        }

        return HttpOkResult::Empty.into();
    }
}
