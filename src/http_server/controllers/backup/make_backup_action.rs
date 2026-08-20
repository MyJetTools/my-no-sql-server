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
        {status_code: 500, description: "At least one namespace has no snapshot on the disk"},
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
    let failed = crate::operations::backup::save_backup(&action.app, true).await;

    // The status code has to mean "there is a snapshot now", and only that: the
    // namespaces are backed up one by one, so a run can write a perfectly valid
    // archive for one of them and fail on the next. Naming the failed ones keeps
    // the caller from having to check /api/Backup/List to find out what
    // happened.
    if !failed.is_empty() {
        return Err(HttpFailResult::as_fatal_error(format!(
            "No snapshot was written for namespace(s): {}",
            failed.join(", ")
        )));
    }

    HttpOutput::Empty.into_ok_result(true).into()
}
