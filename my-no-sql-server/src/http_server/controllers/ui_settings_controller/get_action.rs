use std::sync::Arc;

use my_http_server::macros::*;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};

use crate::app::AppContext;

use super::models::UiSettingsPublicModel;

#[http_route(
    method: "GET",
    route: "/api/UiSettings",
    controller: "UiSettings",
    description: "Returns UI settings stored next to data files. The MCP write password is reported as a boolean flag — its value is never exposed.",
    summary: "Read UI settings",
    result:[
        {status_code: 200, description: "UI settings", model: "UiSettingsPublicModel"},
    ]
)]
pub struct GetUiSettingsAction {
    app: Arc<AppContext>,
}

impl GetUiSettingsAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &GetUiSettingsAction,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let model = super::storage::load(action.app.settings.persistence_dest.as_str()).await;
    let public = UiSettingsPublicModel::from(&model);
    HttpOutput::as_json(public).into_ok_result(false).into()
}
