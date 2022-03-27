use async_trait::async_trait;
use my_http_server_controllers::controllers::{
    actions::{DeleteAction, GetAction, PutAction},
    documentation::{data_types::HttpDataType, out_results::HttpResult, HttpActionDescription},
};
use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};

use crate::{app::AppContext, db_sync::EventSource};

use super::{
    super::super::contracts::{input_params::*, input_params_doc},
    models::{DeleteTableContract, TableContract},
};

pub struct TablesController {
    app: Arc<AppContext>,
}

impl TablesController {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait]
impl GetAction for TablesController {
    fn get_route(&self) -> &str {
        "/Tables/List"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: super::consts::CONTROLLER_NAME,
            description: "Get List of Tables",

            input_params: None,
            results: vec![HttpResult {
                http_code: 200,
                nullable: true,
                description: "List of tables structure".to_string(),
                data_type: TableContract::get_http_data_structure().into_http_data_type_array(),
            }],
        }
        .into()
    }

    async fn handle_request(&self, _ctx: &mut HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let tables = self.app.db.get_tables().await;

        let mut response: Vec<TableContract> = vec![];

        for db_table in &tables {
            response.push(db_table.as_ref().into());
        }

        HttpOutput::as_json(response).into_ok_result(true).into()
    }
}

#[async_trait]
impl PutAction for TablesController {
    fn get_route(&self) -> &str {
        "/Tables/Clean"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: super::consts::CONTROLLER_NAME,
            description: "Clean Table",

            input_params: Some(vec![
                input_params_doc::table_name(),
                input_params_doc::sync_period(),
            ]),
            results: vec![HttpResult {
                http_code: 202,
                nullable: true,
                description: "Table is cleaned".to_string(),
                data_type: HttpDataType::as_string(),
            }],
        }
        .into()
    }

    async fn handle_request(&self, ctx: &mut HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let query = ctx.request.get_query_string()?;

        let table_name = query.get_table_name()?;
        let sync_period = query.get_sync_period();

        let db_table =
            crate::db_operations::read::table::get(self.app.as_ref(), table_name).await?;

        let event_src = EventSource::as_client_request(self.app.as_ref());

        crate::db_operations::write::clean_table::execute(
            self.app.as_ref(),
            db_table,
            event_src,
            sync_period.get_sync_moment(),
        )
        .await;

        return Ok(HttpOutput::Empty.into_ok_result(true));
    }
}

#[async_trait]
impl DeleteAction for TablesController {
    fn get_route(&self) -> &str {
        "/Tables/Delete"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: super::consts::CONTROLLER_NAME,
            description: "Delete Table",

            input_params: DeleteTableContract::get_input_params().into(),
            results: vec![HttpResult {
                http_code: 202,
                nullable: true,
                description: "Table is deleted".to_string(),
                data_type: HttpDataType::as_string(),
            }],
        }
        .into()
    }

    async fn handle_request(&self, ctx: &mut HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let input_data = DeleteTableContract::parse_http_input(ctx).await?;

        if input_data.api_key != self.app.settings.table_api_key.as_str() {
            return Err(HttpFailResult::as_unauthorized(None));
        }

        let event_src = EventSource::as_client_request(self.app.as_ref());

        crate::db_operations::write::table::delete(
            self.app.clone(),
            input_data.table_name,
            event_src,
            DEFAULT_SYNC_PERIOD.get_sync_moment(),
        )
        .await?;

        return Ok(HttpOutput::Empty.into_ok_result(true));
    }
}
