use my_http_server::macros::*;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};
use std::sync::Arc;

use crate::app::AppContext;

#[http_route(
    method: "POST",
    route: "/api/Backup/RestoreFromBackup",
    description: "Restore database from backup folder",
    summary: "Restore database from backup folder",
    controller: "Backup",
    input_data: RestoreFromBackupInputData,
    result:[
        {status_code: 204, description: "Restored ok"},
    ]
)]
pub struct RestoreFromBackupAction {
    app: Arc<AppContext>,
}

impl RestoreFromBackupAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &RestoreFromBackupAction,
    input_data: RestoreFromBackupInputData,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let mut backup_file = action.app.settings.get_backup_folder().to_string();
    if !backup_file.ends_with(std::path::MAIN_SEPARATOR) {
        backup_file.push(std::path::MAIN_SEPARATOR);
    }

    backup_file.push_str(&input_data.file_name);

    let file = tokio::fs::read(backup_file.as_str()).await;

    if let Err(err) = &file {
        return Err(HttpFailResult::as_not_supported_content_type(format!(
            "Error loading file {}. Err:{:?}",
            backup_file, err
        )));
    }

    HttpOutput::Empty.into_ok_result(true).into()
}

#[derive(MyHttpInput)]
pub struct RestoreFromBackupInputData {
    #[http_form_data(name = "fileName", description = "File in backup folder")]
    pub file_name: String,
}
