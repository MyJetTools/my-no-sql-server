use my_http_server::macros::*;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};
use std::sync::Arc;

use crate::app::AppContext;

#[http_route(
    method: "POST",
    route: "/api/Backup/MakeBackup",
    description: "Force creating a snapshot (backup) right now, ignoring the scheduled interval",
    summary: "Force creating a snapshot now",
    controller: "Backup",
    result:[
        {status_code: 204, description: "Snapshot created"},
    ]
)]
pub struct MakeBackupAction {
    app: Arc<AppContext>,
}

impl MakeBackupAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &MakeBackupAction,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    crate::operations::backup::save_backup(&action.app, true).await;
    HttpOutput::Empty.into_ok_result(true).into()
}
