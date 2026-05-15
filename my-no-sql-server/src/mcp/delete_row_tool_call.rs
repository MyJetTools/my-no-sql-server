use std::sync::Arc;

use mcp_server_middleware::*;
use my_ai_agent::macros::ApplyJsonSchema;
use my_no_sql_sdk::core::rust_extensions::date_time::DateTimeAsMicroseconds;
use serde::*;

use crate::{
    app::AppContext,
    db_operations,
    db_sync::{DataSynchronizationPeriod, EventSource},
};

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct DeleteRowInputData {
    #[property(description = "Name of the table")]
    pub table_name: String,
    #[property(description = "Partition key")]
    pub partition_key: String,
    #[property(description = "Row key")]
    pub row_key: String,
    #[property(
        description = "Optional. Only used by MCP clients that do NOT support elicitation. On elicitation-capable clients (e.g. Claude Code) the server will prompt the user interactively and IGNORE this field."
    )]
    pub password: Option<String>,
}

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct DeleteRowResponse {
    #[property(description = "Outcome message")]
    pub status: String,
}

pub struct DeleteRowToolCallHandler {
    app: Arc<AppContext>,
}

impl DeleteRowToolCallHandler {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

impl ToolDefinition for DeleteRowToolCallHandler {
    const FUNC_NAME: &'static str = "delete_row";

    const DESCRIPTION: &'static str = "\
Deletes a single row by partition_key + row_key. Requires the \
mcp-write-password. POLICY: the password is requested via MCP \
elicitation — your client will prompt the user. NEVER cache, log, or \
summarize the password value. See prompt 'mcp_write_password_policy'.";
}

#[async_trait::async_trait]
impl McpToolCallEx<DeleteRowInputData, DeleteRowResponse> for DeleteRowToolCallHandler {
    async fn execute_tool_call(
        &self,
        model: DeleteRowInputData,
        ctx: &ToolCallContext,
    ) -> Result<DeleteRowResponse, String> {
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
        let now = DateTimeAsMicroseconds::now();

        db_operations::write::delete_row::execute(
            self.app.as_ref(),
            &db_table,
            model.partition_key,
            model.row_key,
            event_src,
            DataSynchronizationPeriod::Sec5.get_sync_moment(),
            now,
        )
        .await
        .map_err(|err| format!("{:?}", err))?;

        Ok(DeleteRowResponse {
            status: "deleted".into(),
        })
    }
}
