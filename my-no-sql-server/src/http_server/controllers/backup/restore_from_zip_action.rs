use my_http_server::macros::*;
use my_http_server::types::FileContent;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};
use std::sync::Arc;

use crate::app::AppContext;

#[http_route(
    method: "POST",
    route: "/api/Backup/RestoreFromZip",
    description: "Restore database from backup zip file",
    summary: "Restore database from backup zip file",
    controller: "Backup",
    input_data: RestoreFromBackupZipFileInputData,
    result:[
        {status_code: 204, description: "Restored ok"},
    ]
)]
pub struct RestoreFromZipAction {
    app: Arc<AppContext>,
}

impl RestoreFromZipAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &RestoreFromZipAction,
    input_data: RestoreFromBackupZipFileInputData,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let table_name = input_data.get_table_name().map(|itm| itm.to_string());

    let restore_result = crate::operations::backup::restore(
        &action.app,
        input_data.zip.content,
        table_name.as_deref(),
        input_data.clean_table,
    )
    .await;

    match restore_result {
        Ok(_) => HttpOutput::Empty.into_ok_result(true).into(),
        Err(err) => Err(HttpFailResult::as_fatal_error(format!("{:?}", err))),
    }
}

#[derive(MyHttpInput)]
pub struct RestoreFromBackupZipFileInputData {
    #[http_form_data(
        name = "tableName",
        description = "Name of the table or '*' for all tables"
    )]
    pub table_name: String,

    #[http_form_data(name = "fileName", description = "File in backup folder")]
    pub zip: FileContent,

    #[http_form_data(name = "cleanTable", description = "Clean table before restore")]
    pub clean_table: bool,
}

impl RestoreFromBackupZipFileInputData {
    pub fn get_table_name(&self) -> Option<&str> {
        if self.table_name == "*" {
            None
        } else {
            Some(self.table_name.as_str())
        }
    }
}
