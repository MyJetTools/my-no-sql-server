use std::sync::Arc;

use my_http_server::{
    macros::{http_route, MyHttpInput},
    HttpContext, HttpFailResult, HttpOkResult, HttpOutput,
};
use my_no_sql_sdk::server::rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::app::AppContext;

#[http_route(
    method: "POST",
    route: "/api/Ping",
    controller: "Monitoring",
    description: "Endpoint to ping the service",
    summary: "Endpoint to ping the service",
    input_data: PingHttpInputModel,
    result:[
        {status_code: 204, description: "Ok result"},
    ]
)]
pub struct PingAction {
    app: Arc<AppContext>,
}

impl PingAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &PingAction,
    input_data: PingHttpInputModel,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let now = DateTimeAsMicroseconds::now();

    action
        .app
        .http_writers
        .update(
            &input_data.name,
            &input_data.version,
            input_data.tables.iter().map(|itm| itm.as_str()),
            now,
        )
        .await;
    HttpOutput::Empty.into_ok_result(false).into()
}

#[derive(Debug, MyHttpInput)]
pub struct PingHttpInputModel {
    #[http_body(name = "name", description = "Client Name")]
    pub name: String,
    #[http_body(name = "version", description = "Client Version")]
    pub version: String,

    #[http_body(name = "tables", description = "List of tables with")]
    pub tables: Vec<String>,
}
