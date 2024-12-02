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

    let restore_result = match file {
        Ok(backup_content) => {
            crate::operations::backup::restore(
                &action.app,
                backup_content,
                input_data.get_table_name(),
                input_data.clean_table,
            )
            .await
        }
        Err(err) => {
            return Err(HttpFailResult::as_not_supported_content_type(format!(
                "Error loading file {}. Err:{:?}",
                backup_file, err
            )));
        }
    };

    match restore_result {
        Ok(_) => HttpOutput::Empty.into_ok_result(true).into(),
        Err(err) => Err(HttpFailResult::as_fatal_error(format!("{:?}", err))),
    }
}

#[derive(MyHttpInput)]
pub struct RestoreFromBackupInputData {
    #[http_form_data(
        name = "tableName",
        description = "Name of the table or '*' for all tables"
    )]
    pub table_name: String,

    #[http_form_data(name = "fileName", description = "File in backup folder")]
    pub file_name: String,

    #[http_form_data(name = "cleanTable", description = "Clean table before restore")]
    pub clean_table: bool,
}

impl RestoreFromBackupInputData {
    pub fn get_table_name(&self) -> Option<&str> {
        if self.table_name == "*" {
            None
        } else {
            Some(self.table_name.as_str())
        }
    }
}
