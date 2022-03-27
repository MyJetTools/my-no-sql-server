use crate::{app::AppContext, db_sync::EventSource};
use async_trait::async_trait;
use my_http_server_controllers::controllers::{
    actions::PostAction,
    documentation::{out_results::HttpResult, HttpActionDescription},
};
use std::{result::Result, sync::Arc};

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};

use super::models::{CreateTableCotnract, TableContract};

pub struct CreateIfNotExistsAction {
    app: Arc<AppContext>,
}

impl CreateIfNotExistsAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait]
impl PostAction for CreateIfNotExistsAction {
    fn get_route(&self) -> &str {
        "/Tables/CreateIfNotExists"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: super::consts::CONTROLLER_NAME,
            description: "Migrate records from the other table of other instance",

            input_params: CreateTableCotnract::get_input_params().into(),

            results: vec![HttpResult {
                http_code: 200,
                nullable: true,
                description: "Table structure".to_string(),
                data_type: TableContract::get_http_data_structure().into_http_data_type_object(),
            }],
        }
        .into()
    }

    async fn handle_request(&self, ctx: &mut HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let input_data = CreateTableCotnract::parse_http_input(ctx).await?;

        let even_src = EventSource::as_client_request(self.app.as_ref());

        let table = crate::db_operations::write::table::create_if_not_exist(
            &self.app,
            input_data.table_name.as_str(),
            input_data.persist,
            input_data.max_partitions_amount,
            even_src,
            input_data.sync_period.get_sync_moment(),
        )
        .await?;

        let response: TableContract = table.as_ref().into();

        HttpOutput::as_json(response).into_ok_result(true).into()
    }
}
