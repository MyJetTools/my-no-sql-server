use std::sync::Arc;

use my_http_server::macros::*;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};

use crate::app::AppContext;

use super::models::{UiSettingsPatchBody, UiSettingsPublicModel, UiSettingsUpdateInput};

#[http_route(
    method: "POST",
    route: "/api/UiSettings",
    controller: "UiSettings",
    description: "Partial update of UI settings (warnMs/badMs/mcpWritePassword). Fields omitted from the body are left unchanged; mcpWritePassword set to '' clears the password.",
    summary: "Update UI settings",
    input_data: UiSettingsUpdateInput,
    result:[
        {status_code: 200, description: "Saved settings", model: "UiSettingsPublicModel"},
    ]
)]
pub struct PostUiSettingsAction {
    app: Arc<AppContext>,
}

impl PostUiSettingsAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &PostUiSettingsAction,
    input_data: UiSettingsUpdateInput,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let patch: UiSettingsPatchBody = match serde_json::from_slice(input_data.body.as_slice()) {
        Ok(m) => m,
        Err(err) => {
            return Err(HttpFailResult::as_validation_error(format!(
                "Invalid body: {}",
                err
            )));
        }
    };

    let mut current = super::storage::load(action.app.settings.persistence_dest.as_str()).await;

    if let Some(v) = patch.warn_ms {
        current.warn_ms = v;
    }
    if let Some(v) = patch.bad_ms {
        current.bad_ms = v;
    }
    if let Some(p) = patch.mcp_write_password.as_deref() {
        current.set_mcp_write_password(Some(p));
    }

    let sanitized = current.sanitized();

    if let Err(err) =
        super::storage::save(action.app.settings.persistence_dest.as_str(), &sanitized).await
    {
        return Err(HttpFailResult::as_validation_error(format!(
            "Failed to save settings: {}",
            err
        )));
    }

    let public = UiSettingsPublicModel::from(&sanitized);
    HttpOutput::as_json(public).into_ok_result(false).into()
}
