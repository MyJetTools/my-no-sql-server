use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};

use crate::app::AppContext;

#[my_http_server_swagger::http_route(
    method: "POST",
    route: "/Persist/Force",
    summary: "Execute persist loop",
    description: "Executes persist loop",
    controller: "Persist",
    result:[
        {status_code: 202, description: "Executed succesfully"},
    ]
)]
pub struct ForcePersistAction {
    app: Arc<AppContext>,
}

impl ForcePersistAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &ForcePersistAction,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    crate::operations::persist(&action.app).await;
    HttpOutput::Empty.into_ok_result(true)
}
