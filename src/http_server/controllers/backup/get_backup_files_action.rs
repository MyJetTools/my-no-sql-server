use my_http_server::macros::*;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};
use std::sync::Arc;

use crate::app::AppContext;

#[http_route(
    method: "GET",
    route: "/api/Backup/List",
    description: "Get list of backup files",
    summary: "Get list of backup files",
    controller: "Backup",
    result:[
        {status_code: 200, description: "List of Backup files", model: "Vec<String>"},
    ]
)]
pub struct GetListOfBackupFilesAction {
    app: Arc<AppContext>,
}

impl GetListOfBackupFilesAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &GetListOfBackupFilesAction,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let list_of_files = crate::operations::backup::get_list_of_files(&action.app).await;

    HttpOutput::as_json(list_of_files)
        .into_ok_result(true)
        .into()
}
