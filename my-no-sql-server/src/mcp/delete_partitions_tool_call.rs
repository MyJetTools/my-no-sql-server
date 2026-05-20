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
pub struct DeletePartitionsInputData {
    #[property(description = "Name of the table")]
    pub table_name: String,
    #[property(description = "Partition keys to remove (all rows under each partition will be deleted)")]
    pub partition_keys: Vec<String>,
    #[property(
        description = "Optional: only for clients without elicitation support. On elicitation-capable clients the server will request the password interactively and IGNORE this field. One verification covers the whole batch."
    )]
    pub password: Option<String>,
}

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct DeletePartitionsResponse {
    #[property(description = "Outcome message")]
    pub status: String,
    #[property(description = "Number of partitions submitted for deletion")]
    pub partitions_submitted: usize,
}

pub struct DeletePartitionsToolCallHandler {
    app: Arc<AppContext>,
}

impl DeletePartitionsToolCallHandler {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

impl ToolDefinition for DeletePartitionsToolCallHandler {
    const FUNC_NAME: &'static str = "delete_partitions";

    const DESCRIPTION: &'static str = "\
Deletes whole partitions (and every row they contain) from a table in \
one call. Pass `table_name` and `partition_keys[]`. The mcp-write-password \
is verified ONCE for the whole batch, not per partition. POLICY: the \
password is requested via MCP elicitation — your client will prompt the \
user. NEVER cache, log, or summarize the password value. See prompt \
'mcp_write_password_policy'.";
}

#[async_trait::async_trait]
impl McpToolCallEx<DeletePartitionsInputData, DeletePartitionsResponse>
    for DeletePartitionsToolCallHandler
{
    async fn execute_tool_call(
        &self,
        model: DeletePartitionsInputData,
        ctx: &ToolCallContext,
    ) -> Result<DeletePartitionsResponse, String> {
        if model.partition_keys.is_empty() {
            return Err("`partition_keys` is empty — nothing to delete.".into());
        }

        super::password_check::elicit_or_validate_password(
            self.app.as_ref(),
            ctx,
            model.password.as_deref(),
        )
        .await?;

        let db_table = db_operations::read::table::get(self.app.as_ref(), &model.table_name)
            .await
            .map_err(|err| format!("{:?}", err))?;

        let partitions_submitted = model.partition_keys.len();

        let event_src = EventSource::as_client_request(self.app.as_ref());
        let now = DateTimeAsMicroseconds::now();

        db_operations::write::delete_partitions(
            self.app.as_ref(),
            &db_table,
            model.partition_keys.into_iter(),
            event_src,
            DataSynchronizationPeriod::Sec5.get_sync_moment(),
            now,
        )
        .await
        .map_err(|err| format!("{:?}", err))?;

        Ok(DeletePartitionsResponse {
            status: "ok".into(),
            partitions_submitted,
        })
    }
}
