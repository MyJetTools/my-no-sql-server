use my_http_server::macros::*;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult};
use my_no_sql_sdk::core::rust_extensions::date_time::DateTimeAsMicroseconds;
use std::sync::Arc;

use super::models::*;
use crate::app::AppContext;

#[http_route(
    method: "GET",
    route: "/api/Row",
    deprecated_routes: ["/Row"],
    controller: "Row",
    description: "Get Entity or entities",
    summary: "Returns Entity or entities",
    input_data: "GetRowInputModel",
    result:[
        {status_code: 200, description: "Single Row or array of rows"},
    ]
)]
pub struct GetRowsAction {
    app: Arc<AppContext>,
}

impl GetRowsAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &GetRowsAction,
    input_data: GetRowInputModel,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let db_table =
        crate::db_operations::read::table::get(action.app.as_ref(), input_data.table_name.as_ref())
            .await?;

    let now = DateTimeAsMicroseconds::now();
    if let Some(partition_key) = input_data.partition_key.as_ref() {
        if let Some(row_key) = input_data.row_key.as_ref() {
            let result = crate::db_operations::read::rows::get_single(
                &action.app,
                &db_table,
                partition_key,
                row_key,
                input_data.get_update_statistics(),
                now,
            )
            .await?;

            return Ok(result.into());
        } else {
            let result = crate::db_operations::read::rows::get_all_by_partition_key(
                &action.app,
                &db_table,
                partition_key,
                input_data.limit,
                input_data.skip,
                input_data.get_update_statistics(),
                now,
            )
            .await?;

            return Ok(result.into());
        }
    } else {
        if let Some(row_key) = input_data.row_key.as_ref() {
            let result = crate::db_operations::read::rows::get_all_by_row_key(
                &action.app,
                &db_table,
                row_key,
                input_data.limit,
                input_data.skip,
                input_data.get_update_statistics(),
                now,
            )
            .await?;

            return Ok(result.into());
        } else {
            let result = crate::db_operations::read::rows::get_all(
                &action.app,
                &db_table,
                input_data.limit,
                input_data.skip,
                input_data.get_update_statistics(),
                now,
            )
            .await?;

            return Ok(result.into());
        }
    }
}
