use async_trait::async_trait;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};
use my_http_server_controllers::controllers::{
    actions::{GetAction, PostAction},
    documentation::{data_types::HttpDataType, out_results::HttpResult, HttpActionDescription},
};

use super::super::super::contracts::{input_params::*, input_params_doc, response};
use crate::{app::AppContext, db_sync::EventSource};
use std::{result::Result, sync::Arc};

pub struct TablesController2 {
    app: Arc<AppContext>,
}

impl TablesController2 {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait]
impl GetAction for TablesController2 {
    fn get_route(&self) -> &str {
        "/Tables/PartitionsCount"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: super::consts::CONTROLLER_NAME,
            description: "Get Partitions count",

            input_params: Some(vec![input_params_doc::table_name()]),
            results: vec![
                HttpResult {
                    http_code: 200,
                    nullable: true,
                    description: "Partitions count".to_string(),
                    data_type: HttpDataType::as_long(),
                },
                response::table_not_found(),
            ],
        }
        .into()
    }

    async fn handle_request(&self, ctx: &mut HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let query = ctx.request.get_query_string()?;

        let table_name = query.get_table_name()?;

        let db_table =
            crate::db_operations::read::table::get(self.app.as_ref(), table_name).await?;

        let partitions_amount = db_table.get_partitions_amount().await;

        HttpOutput::as_text(format!("{}", partitions_amount))
            .into_ok_result(true)
            .into()
    }
}

#[async_trait]
impl PostAction for TablesController2 {
    fn get_route(&self) -> &str {
        "/Tables/UpdatePersist"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: super::consts::CONTROLLER_NAME,
            description: "Update table persistence state",

            input_params: Some(vec![
                input_params_doc::table_name(),
                input_params_doc::sync_period(),
            ]),
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
        let query = ctx.request.get_query_string()?;

        let table_name = query.get_table_name()?;

        let persist = query.get_persist_table();

        let max_partitions_amount = query.get_max_partitions_amount();

        let db_table =
            crate::db_operations::read::table::get(self.app.as_ref(), table_name).await?;

        let event_src = EventSource::as_client_request(self.app.as_ref());

        crate::db_operations::write::table::set_table_attrubutes(
            self.app.as_ref(),
            db_table,
            persist,
            max_partitions_amount,
            event_src,
        )
        .await;

        HttpOutput::Empty.into_ok_result(true).into()
    }
}
