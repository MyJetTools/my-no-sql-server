use std::time::Duration;

use mcp_server_middleware::{ElicitationAction, ToolCallContext};
use serde_json::json;

use crate::app::AppContext;

/// Loads the latest persisted UI settings and verifies that the
/// supplied password matches the configured `mcp-write-password`.
/// Returns a tool-friendly error string on any failure — never leaks
/// the input value back.
pub async fn verify_write_password(app: &AppContext, password: &str) -> Result<(), String> {
    let settings = crate::http_server::controllers::ui_settings_controller::storage::load(
        app.settings.persistence_dest.as_str(),
    )
    .await;

    if !settings.has_mcp_write_password() {
        return Err(
            "MCP write password is not configured on the server. Ask the admin to set it on the UI Settings page.".into()
        );
    }

    if !settings.verify_mcp_write_password(password) {
        return Err("Invalid mcp-write-password.".into());
    }

    Ok(())
}

/// Resolves and verifies the write password for a tool call.
///
/// Preferred path: client supports MCP elicitation → server sends
/// `elicitation/create`, user types the password in the client UI.
/// The value never enters the LLM context.
///
/// Fallback: client did not advertise elicitation capability. The
/// tool's optional `password` Input field is used if present;
/// otherwise the call is refused.
pub async fn elicit_or_validate_password(
    app: &AppContext,
    ctx: &ToolCallContext,
    fallback_from_input: Option<&str>,
) -> Result<(), String> {
    let password: String = if ctx.supports_elicitation {
        let schema = json!({
            "type": "object",
            "title": "MCP write password",
            "properties": {
                "password": {
                    "type": "string",
                    "title": "Password",
                    "description": "One-shot value. Do NOT cache."
                }
            },
            "required": ["password"]
        });

        let resp = ctx
            .elicit(
                "This MCP server requires the mcp-write-password to perform a destructive operation. Please type it now.",
                schema,
                Duration::from_secs(120),
            )
            .await?;

        match resp.action {
            ElicitationAction::Accept => {
                let content = resp
                    .content
                    .ok_or_else(|| "Elicitation accept without content".to_string())?;
                content
                    .get("password")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Elicitation accept missing 'password' field".to_string())?
                    .to_string()
            }
            ElicitationAction::Decline => {
                return Err("User declined to provide the write password.".into());
            }
            ElicitationAction::Cancel => {
                return Err("User cancelled the write operation.".into());
            }
        }
    } else {
        match fallback_from_input {
            Some(p) if !p.is_empty() => p.to_string(),
            _ => {
                return Err(
                    "MCP write password required. Either use a client that supports MCP elicitation (e.g. Claude Code), or pass `password` in the tool arguments.".into()
                );
            }
        }
    };

    verify_write_password(app, &password).await
}
