use std::sync::Arc;

use my_http_server::middlewares::controllers::actions::PostAction;
use my_http_server::middlewares::controllers::documentation::data_types::HttpDataType;
use my_http_server::middlewares::controllers::documentation::out_results::HttpResult;
use my_http_server::middlewares::controllers::documentation::HttpActionDescription;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult};

use crate::db_json_entity::{DbJsonEntity, JsonTimeStamp};

use crate::app::AppContext;

use super::models::BulkInsertOrReplaceInputContract;

pub struct BlukInsertOrReplaceControllerAction {
    app: Arc<AppContext>,
}

impl BlukInsertOrReplaceControllerAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl PostAction for BlukInsertOrReplaceControllerAction {
    fn get_route(&self) -> &str {
        "/Bulk/InsertOrReplace"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: super::consts::CONTROLLER_NAME,
            description: "Bulk insert or replace operation",

            input_params: BulkInsertOrReplaceInputContract::get_input_params().into(),
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
        let input_data = BulkInsertOrReplaceInputContract::parse_http_input(ctx).await?;

        let db_table = crate::db_operations::read::table::get(
            self.app.as_ref(),
            input_data.table_name.as_str(),
        )
        .await?;

        let attr = crate::operations::transaction_attributes::create(
            self.app.as_ref(),
            input_data.sync_period,
        );

        let now = JsonTimeStamp::now();

        let rows_by_partition = DbJsonEntity::parse_as_btreemap(input_data.body.as_slice(), &now)?;

        crate::db_operations::write::bulk_insert_or_update::execute(
            self.app.as_ref(),
            db_table,
            rows_by_partition,
            Some(attr),
            &now,
        )
        .await;

        HttpOkResult::Empty.into()
    }
}
