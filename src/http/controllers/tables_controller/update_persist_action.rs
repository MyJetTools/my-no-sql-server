use async_trait::async_trait;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};
use my_http_server_controllers::controllers::{
    actions::PostAction,
    documentation::{data_types::HttpDataType, out_results::HttpResult, HttpActionDescription},
};

use super::{super::super::contracts::response, models::UpdatePersistTableContract};
use crate::{app::AppContext, db_sync::EventSource};
use std::{result::Result, sync::Arc};

pub struct UpdatePersistAction {
    app: Arc<AppContext>,
}

impl UpdatePersistAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait]
impl PostAction for UpdatePersistAction {
    fn get_route(&self) -> &str {
        "/Tables/UpdatePersist"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: super::consts::CONTROLLER_NAME,
            description: "Update table persistence state",

            input_params: UpdatePersistTableContract::get_input_params().into(),
            results: vec![
                HttpResult {
                    http_code: 202,
                    nullable: true,
                    description: "Persist state is updated".to_string(),
                    data_type: HttpDataType::as_string(),
                },
                response::table_not_found(),
            ],
        }
        .into()
    }

    async fn handle_request(&self, ctx: &mut HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let input_data = UpdatePersistTableContract::parse_http_input(ctx).await?;

        let db_table = crate::db_operations::read::table::get(
            self.app.as_ref(),
            input_data.table_name.as_str(),
        )
        .await?;

        let event_src = EventSource::as_client_request(self.app.as_ref());

        crate::db_operations::write::table::update_persist_state(
            &self.app,
            db_table,
            input_data.persist,
            event_src,
        )
        .await;

        HttpOutput::Empty.into_ok_result(true).into()
    }
}
