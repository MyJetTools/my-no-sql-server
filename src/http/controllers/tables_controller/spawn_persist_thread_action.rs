use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};

use crate::app::AppContext;

use super::models::SpawnPersistThreadInputContract;

#[my_http_server_swagger::http_route(
    method: "POST",
    route: "/Tables/SpawnPersistThread",
    input_data: "SpawnPersistThreadInputContract",
    description: "Get Table size",
    controller: "Tables",
)]
pub struct SpawnPersistThreadAction {
    app: Arc<AppContext>,
}

impl SpawnPersistThreadAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &SpawnPersistThreadAction,
    input_data: SpawnPersistThreadInputContract,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    crate::db_operations::check_app_states(action.app.as_ref())?;

    let db_table =
        crate::db_operations::read::table::get(action.app.as_ref(), input_data.table_name.as_str())
            .await?;

    let result = crate::operations::spawn_dedicated_persist_thread(&action.app, db_table).await;

    match result {
        Ok(_) => HttpOutput::as_text("Thread is spawned".to_string())
            .into_ok_result(true)
            .into(),
        Err(msg) => HttpOutput::as_text(format!("Can not spawn thread. Reason:{}", msg))
            .into_ok_result(true)
            .into(),
    }
}
