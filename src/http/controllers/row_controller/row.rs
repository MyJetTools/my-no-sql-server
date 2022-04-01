use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult};
use my_http_server_controllers::controllers::actions::{DeleteAction, GetAction, PutAction};
use my_http_server_controllers::controllers::documentation::out_results::HttpResult;
use my_http_server_controllers::controllers::documentation::HttpActionDescription;

use crate::db::UpdateExpirationTimeModel;
use crate::db_json_entity::{DbJsonEntity, JsonTimeStamp};

use crate::db_operations;

use crate::app::AppContext;
use crate::db_sync::EventSource;

use super::models::{
    BaseDbRowContract, DeleteRowInputModel, GetRowInputModel, ReplaceInputContract,
};

pub struct RowAction {
    app: Arc<AppContext>,
}

impl RowAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl GetAction for RowAction {
    fn get_route(&self) -> &str {
        "/Row"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: super::consts::CONTROLLER_NAME,
            description: "Get Entities",

            input_params: GetRowInputModel::get_input_params().into(),
            results: vec![
                HttpResult {
                    http_code: 200,
                    nullable: false,
                    description: "Rows".to_string(),
                    data_type: BaseDbRowContract::get_http_data_structure()
                        .into_http_data_type_array(),
                },
                crate::http::docs::rejects::op_with_table_is_failed(),
            ],
        }
        .into()
    }

    async fn handle_request(&self, ctx: &mut HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let input_data = GetRowInputModel::parse_http_input(ctx).await?;

        let db_table = crate::db_operations::read::table::get(
            self.app.as_ref(),
            input_data.table_name.as_ref(),
        )
        .await?;

        let update_expiration = UpdateExpirationTimeModel::new(
            input_data.set_db_rows_expiration_time.as_ref(),
            input_data.set_partition_expiration_time.as_ref(),
        );

        if let Some(partition_key) = input_data.partition_key.as_ref() {
            if let Some(row_key) = input_data.row_key.as_ref() {
                let result = crate::db_operations::read::rows::get_single(
                    db_table.as_ref(),
                    partition_key,
                    row_key,
                    update_expiration,
                )
                .await?;

                return Ok(result.into());
            } else {
                let result = crate::db_operations::read::rows::get_all_by_partition_key(
                    db_table.as_ref(),
                    partition_key,
                    input_data.limit,
                    input_data.skip,
                    update_expiration,
                )
                .await;

                return Ok(result.into());
            }
        } else {
            if let Some(row_key) = input_data.row_key.as_ref() {
                let result = crate::db_operations::read::rows::get_all_by_row_key(
                    db_table.as_ref(),
                    row_key,
                    input_data.limit,
                    input_data.skip,
                    update_expiration,
                )
                .await;

                return Ok(result.into());
            } else {
                let result = crate::db_operations::read::rows::get_all(
                    db_table.as_ref(),
                    input_data.limit,
                    input_data.skip,
                    update_expiration,
                )
                .await;

                return Ok(result.into());
            }
        }
    }
}

#[async_trait::async_trait]
impl DeleteAction for RowAction {
    fn get_route(&self) -> &str {
        "/Row"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: super::consts::CONTROLLER_NAME,
            description: "Delete Entitiy",

            input_params: DeleteRowInputModel::get_input_params().into(),
            results: vec![
                HttpResult {
                    http_code: 200,
                    nullable: false,
                    description: "Deleted row".to_string(),
                    data_type: BaseDbRowContract::get_http_data_structure()
                        .into_http_data_type_object(),
                },
                crate::http::docs::rejects::op_with_table_is_failed(),
            ],
        }
        .into()
    }

    async fn handle_request(&self, ctx: &mut HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let http_input = DeleteRowInputModel::parse_http_input(ctx).await?;

        let db_table = crate::db_operations::read::table::get(
            self.app.as_ref(),
            http_input.table_name.as_ref(),
        )
        .await?;

        let event_src = EventSource::as_client_request(self.app.as_ref());

        let now = JsonTimeStamp::now();
        let result: HttpOkResult = db_operations::write::delete_row::execute(
            self.app.as_ref(),
            db_table,
            http_input.partition_key.as_ref(),
            &http_input.row_key,
            event_src,
            &now,
            http_input.sync_period.get_sync_moment(),
        )
        .await
        .into();

        result.into()
    }
}

#[async_trait::async_trait]
impl PutAction for RowAction {
    fn get_route(&self) -> &str {
        "/Row/Replace"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: super::consts::CONTROLLER_NAME,
            description: "Replace Entitiy",

            input_params: DeleteRowInputModel::get_input_params().into(),
            results: vec![
                HttpResult {
                    http_code: 200,
                    nullable: false,
                    description: "Replaced row".to_string(),
                    data_type: BaseDbRowContract::get_http_data_structure()
                        .into_http_data_type_object(),
                },
                crate::http::docs::rejects::op_with_table_is_failed(),
            ],
        }
        .into()
    }

    async fn handle_request(&self, ctx: &mut HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let input_data = ReplaceInputContract::parse_http_input(ctx).await?;

        let db_table = crate::db_operations::read::table::get(
            self.app.as_ref(),
            input_data.table_name.as_ref(),
        )
        .await?;

        let now = JsonTimeStamp::now();

        let db_json_entity = DbJsonEntity::parse(input_data.body.as_ref())?;

        crate::db_operations::write::replace::validate_before(
            db_table.as_ref(),
            db_json_entity.partition_key,
            db_json_entity.row_key,
            db_json_entity.time_stamp,
        )
        .await?;

        let db_row = Arc::new(db_json_entity.to_db_row(&now));

        let event_src = EventSource::as_client_request(self.app.as_ref());

        let result: HttpOkResult = crate::db_operations::write::replace::execute(
            self.app.as_ref(),
            db_table.as_ref(),
            db_json_entity.partition_key,
            db_row,
            event_src,
            db_json_entity.time_stamp.unwrap(),
            &now,
            input_data.sync_period.get_sync_moment(),
        )
        .await?
        .into();

        result.into()
    }
}
