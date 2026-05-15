use std::sync::Arc;

use mcp_server_middleware::*;
use my_ai_agent::macros::ApplyJsonSchema;
use serde::*;

use crate::app::AppContext;

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct GetListOfTablesInputData {}

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct GetListOfTablesResponse {
    #[property(description = "Amount of tables")]
    pub count: usize,
    #[property(description = "List of table names")]
    pub tables: Vec<String>,
}

pub struct GetListOfTablesToolCallHandler {
    app: Arc<AppContext>,
}

impl GetListOfTablesToolCallHandler {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

impl ToolDefinition for GetListOfTablesToolCallHandler {
    const FUNC_NAME: &'static str = "get_list_of_tables";

    const DESCRIPTION: &'static str = "Returns the list of all MyNoSql table names available on this server.";
}

#[async_trait::async_trait]
impl McpToolCall<GetListOfTablesInputData, GetListOfTablesResponse>
    for GetListOfTablesToolCallHandler
{
    async fn execute_tool_call(
        &self,
        _model: GetListOfTablesInputData,
    ) -> Result<GetListOfTablesResponse, String> {
        let tables = self.app.db.get_tables();

        let tables: Vec<String> = tables
            .iter()
            .map(|table| table.name.to_string())
            .collect();

        Ok(GetListOfTablesResponse {
            count: tables.len(),
            tables,
        })
    }
}
