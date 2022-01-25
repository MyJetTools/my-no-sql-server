use std::sync::Arc;

use my_http_server::{
    middlewares::controllers::{
        actions::GetAction,
        documentation::{data_types::HttpDataType, out_results::HttpResult, HttpActionDescription},
    },
    HttpContext, HttpFailResult, HttpOkResult,
};

use crate::app::AppContext;

use super::models::RowsCountInputContract;

pub struct RowCountAction {
    app: Arc<AppContext>,
}

impl RowCountAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl GetAction for RowCountAction {
    fn get_route(&self) -> &str {
        "/Count"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: super::consts::CONTROLLER_NAME,
            description: "Get Rows Count",

            input_params: RowsCountInputContract::get_input_params().into(),
            results: vec![HttpResult {
                http_code: 200,
                nullable: false,
                description: "Amount of rows of the table or the partition".to_string(),
                data_type: HttpDataType::as_long(),
            }],
        }
        .into()
    }

    async fn handle_request(&self, ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let input_data = RowsCountInputContract::parse_http_input(ctx).await?;

        let db_table = crate::db_operations::read::table::get(
            self.app.as_ref(),
            input_data.table_name.as_str(),
        )
        .await?;

        if let Some(partition_key) = input_data.partition_key {
            let table_access = db_table.data.read().await;

            let partition = table_access.get_partition(partition_key.as_str());

            if let Some(partition) = partition {
                return HttpOkResult::Text {
                    text: partition.rows_count().to_string(),
                }
                .into();
            } else {
                return Ok(HttpOkResult::Empty);
            }
        }

        let table_access = db_table.data.read().await;

        let mut result = 0;

        for partition in table_access.get_partitions() {
            result += partition.rows_count();
        }
        return HttpOkResult::Text {
            text: result.to_string(),
        }
        .into();
    }
}