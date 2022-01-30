use std::{collections::HashMap, sync::Arc};

use my_http_server::middlewares::controllers::actions::PostAction;
use my_http_server::middlewares::controllers::documentation::data_types::HttpDataType;
use my_http_server::middlewares::controllers::documentation::out_results::HttpResult;
use my_http_server::middlewares::controllers::documentation::HttpActionDescription;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult};

use crate::db_json_entity::JsonTimeStamp;

use crate::app::AppContext;
use crate::db_sync::EventSource;

use super::models::BulkDeleteInputContract;

pub struct BulkDeleteControllerAction {
    app: Arc<AppContext>,
}

impl BulkDeleteControllerAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl PostAction for BulkDeleteControllerAction {
    fn get_route(&self) -> &str {
        "/Bulk/Delete"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: super::consts::CONTROLLER_NAME,
            description: "Bulk delete operation",

            input_params: BulkDeleteInputContract::get_input_params().into(),
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
        let input_data = BulkDeleteInputContract::parse_http_input(ctx).await?;

        let db_table = crate::db_operations::read::table::get(
            self.app.as_ref(),
            input_data.table_name.as_str(),
        )
        .await?;

        let event_src = EventSource::as_client_request(self.app.as_ref(), input_data.sync_period);

        let rows_to_delete: HashMap<String, Vec<String>> =
            serde_json::from_slice(input_data.body.as_slice()).unwrap();

        let now = JsonTimeStamp::now();

        crate::db_operations::write::bulk_delete::execute(
            self.app.as_ref(),
            db_table.as_ref(),
            rows_to_delete,
            Some(event_src),
            &now,
        )
        .await;

        HttpOkResult::Empty.into()
    }
}
