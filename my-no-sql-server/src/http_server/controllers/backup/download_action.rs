use my_http_server::macros::*;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};
use my_no_sql_sdk::core::rust_extensions::date_time::DateTimeAsMicroseconds;
use std::sync::Arc;

use crate::app::AppContext;

#[http_route(
    method: "GET",
    route: "/api/Backup/Download",
    description: "Download all tables as Zip Archive",
    summary: "Download all tables as Zip Archive",
    controller: "Backup",
    result:[
        {status_code: 200, description: "Snapshot of all tables"},
    ]
)]
pub struct DownloadAction {
    app: Arc<AppContext>,
}

impl DownloadAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &DownloadAction,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let db_snapshot_as_zip = crate::operations::build_db_snapshot_as_zip_archive(&action.app).await;

    let now = DateTimeAsMicroseconds::now();

    let filename = format!("{}.zip", &now.to_rfc3339().replace(":", "_")[..19]);

    HttpOutput::as_file(filename.to_string(), db_snapshot_as_zip)
        .into_ok_result(true)
        .into()
}
