use std::sync::Arc;

use my_http_server_controllers::controllers::{
    actions::PostAction,
    documentation::{data_types::HttpDataType, out_results::HttpResult, HttpActionDescription},
};

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};

use crate::{app::AppContext, db_sync::EventSource};

use super::{super::super::contracts::response, models::CreateTableCotnract};

pub struct CreateTableAction {
    app: Arc<AppContext>,
}

impl CreateTableAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl PostAction for CreateTableAction {
    fn get_route(&self) -> &str {
        "/Tables/Create"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: super::consts::CONTROLLER_NAME,
            description: "Create Table",

            input_params: CreateTableCotnract::get_input_params().into(),
            results: vec![
                HttpResult {
                    http_code: 202,
                    nullable: true,
                    description: "Table is created".to_string(),
                    data_type: HttpDataType::as_string(),
                },
                response::table_not_found(),
            ],
        }
        .into()
    }

    async fn handle_request(&self, ctx: &mut HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let input_data = CreateTableCotnract::parse_http_input(ctx).await?;

        let even_src = EventSource::as_client_request(self.app.as_ref());

        crate::db_operations::write::table::create(
            self.app.as_ref(),
            input_data.table_name.as_str(),
            input_data.persist,
            input_data.max_partitions_amount,
            even_src,
            input_data.sync_period.get_sync_moment(),
        )
        .await?;

        return Ok(HttpOutput::Empty.into_ok_result(true));
    }
}
