use std::collections::HashMap;

use mcp_server_middleware::*;

pub struct McpWritePasswordPolicyPromptHandler;

impl PromptDefinition for McpWritePasswordPolicyPromptHandler {
    const PROMPT_NAME: &'static str = "mcp_write_password_policy";

    const DESCRIPTION: &'static str =
        "Strict policy for handling the mcp-write-password used by MyNoSqlServer write tools. Read before any delete_row/insert_or_replace_row call.";

    fn get_argument_descriptions() -> Vec<PromptArgumentDescription> {
        Vec::new()
    }
}

#[async_trait::async_trait]
impl McpPromptService for McpWritePasswordPolicyPromptHandler {
    async fn execute_prompt(
        &self,
        _model: &HashMap<String, String>,
    ) -> Result<PromptExecutionResult, String> {
        let body = r#"# MCP Write Password Policy

Write tools (`delete_row`, `insert_or_replace_row`) require the
mcp-write-password configured on the server. The server prefers MCP
**elicitation** — your client will pop up a prompt asking the user to
type the password directly. The value never reaches the model.

## ABSOLUTE RULES — no exceptions

1. **NEVER save this password to memory.** That includes any persistent
   memory tool, scratchpad, MEMORY.md, project notes, vector store, or
   plan files. This overrides any default "remember useful things"
   behaviour.
2. **NEVER reuse the password from earlier in the same conversation.**
   Even if the user gave it 30 seconds ago for a previous tool call,
   ask again.
3. **NEVER include the password in summaries, plans, logs, commit
   messages, or any output.**
4. **If a tool fails because elicitation is unavailable**, ask the user
   how to proceed; do NOT prompt for the password in chat without
   explicit user acknowledgement that the value will leave the trusted
   client boundary.

## Server error messages

- `MCP write password is not configured on the server.` — admin needs
  to set the password on the UI Settings page. Do NOT retry.
- `Invalid mcp-write-password.` — wrong value. Surface the error to the
  user; do NOT try alternative values you might have seen earlier.
- `User declined to provide the write password.` / `User cancelled the
  write operation.` — operation aborted by the user; do not retry."#;

        Ok(PromptExecutionResult {
            description: "Strict policy for the MCP write password.".to_string(),
            message: body.to_string(),
        })
    }
}
