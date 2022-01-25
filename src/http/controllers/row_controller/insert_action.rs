use std::sync::Arc;

use my_http_server::middlewares::controllers::actions::PostAction;
use my_http_server::middlewares::controllers::documentation::data_types::HttpDataType;
use my_http_server::middlewares::controllers::documentation::out_results::HttpResult;
use my_http_server::middlewares::controllers::documentation::HttpActionDescription;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult};

use crate::db_json_entity::{DbJsonEntity, JsonTimeStamp};

use crate::app::AppContext;

use super::models::InsertInputContract;

use crate::http::docs;

pub struct InsertRowAction {
    app: Arc<AppContext>,
}

impl InsertRowAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}
#[async_trait::async_trait]
impl PostAction for InsertRowAction {
    fn get_route(&self) -> &str {
        "/Row/Insert"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: super::consts::CONTROLLER_NAME,
            description: "Insert Row",

            input_params: InsertInputContract::get_input_params().into(),
            results: vec![
                HttpResult {
                    http_code: 202,
                    nullable: false,
                    description: "Insert operation performed succesfully".to_string(),
                    data_type: HttpDataType::None,
                },
                docs::rejects::op_with_table_is_failed(),
            ],
        }
        .into()
    }

    async fn handle_request(&self, ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let input_data = InsertInputContract::parse_http_input(ctx).await?;

        let db_table = crate::db_operations::read::table::get(
            self.app.as_ref(),
            input_data.table_name.as_str(),
        )
        .await?;
        let db_json_entity = DbJsonEntity::parse(input_data.body.as_slice())?;

        crate::db_operations::write::insert::validate_before(
            db_table.as_ref(),
            db_json_entity.partition_key,
            db_json_entity.row_key,
        )
        .await?;

        let attr = crate::operations::transaction_attributes::create(
            self.app.as_ref(),
            input_data.sync_period,
        );

        let now = JsonTimeStamp::now();

        let db_row = Arc::new(db_json_entity.to_db_row(&now));

        crate::db_operations::write::insert::execute(
            self.app.as_ref(),
            db_table.as_ref(),
            db_row,
            Some(attr),
            &now,
        )
        .await?;

        HttpOkResult::Empty.into()
    }
}
