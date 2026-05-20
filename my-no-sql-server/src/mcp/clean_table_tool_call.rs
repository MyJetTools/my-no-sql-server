use std::sync::Arc;

use mcp_server_middleware::*;
use my_ai_agent::macros::ApplyJsonSchema;
use serde::*;

use crate::{
    app::AppContext,
    db_operations,
    db_sync::{DataSynchronizationPeriod, EventSource},
};

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct CleanTableInputData {
    #[property(description = "Name of the table to clean (all rows in all partitions will be removed)")]
    pub table_name: String,
    #[property(
        description = "Optional: only for clients without elicitation support. On elicitation-capable clients the server will request the password interactively and IGNORE this field."
    )]
    pub password: Option<String>,
}

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct CleanTableResponse {
    #[property(description = "Outcome message")]
    pub status: String,
}

pub struct CleanTableToolCallHandler {
    app: Arc<AppContext>,
}

impl CleanTableToolCallHandler {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

impl ToolDefinition for CleanTableToolCallHandler {
    const FUNC_NAME: &'static str = "clean_table";

    const DESCRIPTION: &'static str = "\
Removes ALL rows from the specified table (the table itself is kept). \
This is a destructive operation and requires the mcp-write-password. \
POLICY: the password is requested via MCP elicitation — your client \
will prompt the user. NEVER cache, log, or summarize the password \
value. See prompt 'mcp_write_password_policy'.";
}

#[async_trait::async_trait]
impl McpToolCallEx<CleanTableInputData, CleanTableResponse> for CleanTableToolCallHandler {
    async fn execute_tool_call(
        &self,
        model: CleanTableInputData,
        ctx: &ToolCallContext,
    ) -> Result<CleanTableResponse, String> {
        super::password_check::elicit_or_validate_password(
            self.app.as_ref(),
            ctx,
            model.password.as_deref(),
        )
        .await?;

        let db_table = db_operations::read::table::get(self.app.as_ref(), &model.table_name)
            .await
            .map_err(|err| format!("{:?}", err))?;

        let event_src = EventSource::as_client_request(self.app.as_ref());

        db_operations::write::clean_table(
            self.app.as_ref(),
            &db_table,
            event_src,
            DataSynchronizationPeriod::Sec5.get_sync_moment(),
        )
        .await
        .map_err(|err| format!("{:?}", err))?;

        Ok(CleanTableResponse {
            status: "cleaned".into(),
        })
    }
}
