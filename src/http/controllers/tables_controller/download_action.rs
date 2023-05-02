use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{app::AppContext, zip::DbZipBuilder};

#[my_http_server_swagger::http_route(
    method: "GET",
    route: "/Tables/Download",
    description: "Download all tables as Zip Archive",
    summary: "Download all tables as Zip Archive",
    controller: "Tables",
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
    crate::db_operations::check_app_states(action.app.as_ref())?;
    let tables = action.app.db.get_tables().await;

    let mut zip_builder = DbZipBuilder::new();

    for db_table in &tables {
        let table_snapshot = db_table.get_table_snapshot().await;

        zip_builder
            .add_table(&db_table.name, &table_snapshot)
            .unwrap();
    }

    let now = DateTimeAsMicroseconds::now();

    HttpOutput::as_file(
        format!("{}.zip", now.to_rfc3339()),
        zip_builder.get_payload().unwrap(),
    )
    .into_ok_result(true)
    .into()
}
