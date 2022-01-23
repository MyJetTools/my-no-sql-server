use crate::{
    app::AppContext,
    http::contracts::{input_params::MyNoSqlQueryString, input_params_doc},
};
use async_trait::async_trait;
use std::{result::Result, sync::Arc};

use my_http_server::{
    middlewares::controllers::{
        actions::PostAction,
        documentation::{
            data_types::HttpObjectStructure, out_results::HttpResult, HttpActionDescription,
        },
    },
    HttpContext, HttpFailResult, HttpOkResult,
};

use super::models::TableContract;

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
    fn get_additional_types(&self) -> Option<Vec<HttpObjectStructure>> {
        None
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: super::consts::CONTROLLER_NAME,
            description: "Migrate records from the other table of other instance",

            input_params: vec![
                input_params_doc::table_name(),
                input_params_doc::max_partitions_amount(),
            ]
            .into(),
            results: vec![HttpResult {
                http_code: 200,
                nullable: true,
                description: "Table structure".to_string(),
                data_type: TableContract::get_http_data_structure().into_http_data_type_object(),
            }],
        }
        .into()
    }

    async fn handle_request(&self, ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let query = ctx.get_query_string()?;

        let table_name = query.get_table_name()?;
        let persist_table = query.get_persist_table();

        let max_partitions_amount = query.get_max_partitions_amount();

        let sync_period = query.get_sync_period();

        let attr =
            crate::operations::transaction_attributes::create(self.app.as_ref(), sync_period);

        let table = crate::db_operations::write::table::create_if_not_exist(
            self.app.as_ref(),
            table_name,
            persist_table,
            max_partitions_amount,
            Some(attr),
        )
        .await;

        let response: TableContract = table.as_ref().into();

        return HttpOkResult::create_json_response(response).into();
    }
}
