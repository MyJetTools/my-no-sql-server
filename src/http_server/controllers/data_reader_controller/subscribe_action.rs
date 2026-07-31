use crate::{app::AppContext, http_server::http_sessions::*};
use my_http_server::macros::*;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};
use std::sync::Arc;

use super::models::SubscribeToTableInputModel;

#[http_route(
    method: "POST",
    route: "/api/DataReader/Subscribe",
    deprecated_routes: ["/DataReader/Subscribe"],
    controller: "DataReader",
    summary: "Subscribes to table",
    description: "Subscribe to table",
    input_data: "SubscribeToTableInputModel",
    result:[
        {status_code: 202, description: "Successful operation"},
    ]
)]
pub struct SubscribeAction {
    app: Arc<AppContext>,
}

impl SubscribeAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &SubscribeAction,
    input_data: SubscribeToTableInputModel,
    ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let db_namespace = crate::http_server::get_request_namespace(&action.app, ctx).await?;

    let data_reader = action
        .app
        .get_http_session(input_data.session_id.as_str())
        .await?;

    crate::operations::data_readers::subscribe(
        &action.app,
        &db_namespace,
        data_reader,
        input_data.table_name.as_str(),
    )
    .await?;

    HttpOutput::Empty.into_ok_result(true).into()
}
