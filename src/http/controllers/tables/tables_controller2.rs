use async_trait::async_trait;
use my_http_server::{
    middlewares::controllers::{
        actions::{GetAction, PostAction},
        documentation::{
            data_types::{HttpDataType, HttpObjectStructure},
            out_results::HttpResult,
            HttpActionDescription,
        },
    },
    HttpContext, HttpFailResult, HttpOkResult,
};

use super::super::super::contracts::{input_params::*, input_params_doc, response};
use crate::app::AppContext;
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
    fn get_additional_types(&self) -> Option<Vec<HttpObjectStructure>> {
        None
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

    async fn handle_request(&self, ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let query = ctx.get_query_string()?;

        let table_name = query.get_table_name()?;

        let db_table =
            crate::db_operations::read::table::get(self.app.as_ref(), table_name).await?;

        let partitions_amount = db_table.get_partitions_amount().await;

        return Ok(HttpOkResult::Text {
            text: format!("{}", partitions_amount),
        });
    }
}

#[async_trait]
impl PostAction for TablesController2 {
    fn get_additional_types(&self) -> Option<Vec<HttpObjectStructure>> {
        None
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

    async fn handle_request(&self, ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let query = ctx.get_query_string()?;

        let table_name = query.get_table_name()?;
        let sync_period = query.get_sync_period();

        let persist = query.get_persist_table();

        let max_partitions_amount = query.get_max_partitions_amount();

        let db_table =
            crate::db_operations::read::table::get(self.app.as_ref(), table_name).await?;

        let attr =
            crate::operations::transaction_attributes::create(self.app.as_ref(), sync_period);

        crate::db_operations::write::table::set_table_attrubutes(
            self.app.as_ref(),
            db_table,
            persist,
            max_partitions_amount,
            Some(attr),
        )
        .await;

        return Ok(HttpOkResult::Empty);
    }
}
