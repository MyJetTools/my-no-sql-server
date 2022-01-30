use std::sync::Arc;

use my_http_server::middlewares::controllers::actions::PostAction;
use my_http_server::middlewares::controllers::documentation::out_results::HttpResult;
use my_http_server::middlewares::controllers::documentation::HttpActionDescription;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult};

use crate::db_json_entity::{DbJsonEntity, JsonTimeStamp};

use crate::app::AppContext;
use crate::db_sync::EventSource;

use super::models::{BaseDbRowContract, InsertOrReplaceInputContract};
use crate::http::docs;

pub struct InsertOrReplaceAction {
    app: Arc<AppContext>,
}

impl InsertOrReplaceAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl PostAction for InsertOrReplaceAction {
    fn get_route(&self) -> &str {
        "/Row/InsertOrReplace"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: super::consts::CONTROLLER_NAME,
            description: "Insert or replace DbEntity",

            input_params: InsertOrReplaceInputContract::get_input_params().into(),
            results: vec![
                HttpResult {
                    http_code: 200,
                    nullable: false,
                    description: "Removed entity".to_string(),
                    data_type: BaseDbRowContract::get_http_data_structure()
                        .into_http_data_type_object(),
                },
                docs::rejects::op_with_table_is_failed(),
            ],
        }
        .into()
    }

    async fn handle_request(&self, ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let input_data = InsertOrReplaceInputContract::parse_http_input(ctx).await?;

        let db_table = crate::db_operations::read::table::get(
            self.app.as_ref(),
            input_data.table_name.as_str(),
        )
        .await?;

        let event_src = EventSource::as_client_request(self.app.as_ref(), input_data.sync_period);

        let now = JsonTimeStamp::now();

        let db_json_entity = DbJsonEntity::parse(input_data.body.as_slice())?;

        let db_row = Arc::new(db_json_entity.to_db_row(&now));

        let result: HttpOkResult = crate::db_operations::write::insert_or_replace::execute(
            self.app.as_ref(),
            db_table,
            db_row,
            Some(event_src),
            &now,
        )
        .await
        .into();

        result.into()
    }
}
