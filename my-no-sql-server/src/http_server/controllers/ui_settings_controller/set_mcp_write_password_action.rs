use std::sync::Arc;

use my_http_server::macros::*;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};

use crate::app::AppContext;

use super::models::{McpWritePasswordBody, McpWritePasswordInput, SettingsPublicModel};

#[http_route(
    method: "POST",
    route: "/api/Settings/McpWritePassword",
    controller: "Settings",
    description: "Sets or clears the MCP write password used by `delete_row` and `insert_or_replace_row` MCP tools. The value is hashed (salt + SHA-256) before persistence — plaintext is never stored or returned. Pass an empty string to clear.",
    summary: "Set MCP write password",
    input_data: McpWritePasswordInput,
    result:[
        {status_code: 200, description: "Updated settings", model: "SettingsPublicModel"},
    ]
)]
pub struct SetMcpWritePasswordAction {
    app: Arc<AppContext>,
}

impl SetMcpWritePasswordAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &SetMcpWritePasswordAction,
    input_data: McpWritePasswordInput,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let body: McpWritePasswordBody = match serde_json::from_slice(input_data.body.as_slice()) {
        Ok(b) => b,
        Err(err) => {
            return Err(HttpFailResult::as_validation_error(format!(
                "Invalid body: {}",
                err
            )));
        }
    };

    let mut current = super::storage::load(action.app.settings.persistence_dest.as_str()).await;
    current.set_mcp_write_password(Some(body.password.as_str()));
    let sanitized = current.sanitized();

    if let Err(err) =
        super::storage::save(action.app.settings.persistence_dest.as_str(), &sanitized).await
    {
        return Err(HttpFailResult::as_validation_error(format!(
            "Failed to save settings: {}",
            err
        )));
    }

    HttpOutput::as_json(SettingsPublicModel::from(&sanitized))
        .into_ok_result(false)
        .into()
}
