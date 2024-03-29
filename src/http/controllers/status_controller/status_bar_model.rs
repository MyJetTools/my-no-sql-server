use crate::app::AppContext;
use my_http_server::macros::*;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct LocationModel {
    pub id: String,
    pub compress: bool,
}

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct StatusBarModel {
    pub location: LocationModel,
    #[serde(rename = "persistAmount")]
    persist_amount: usize,
    #[serde(rename = "tcpConnections")]
    pub tcp_connections: usize,
    #[serde(rename = "tablesAmount")]
    pub tables_amount: usize,
    #[serde(rename = "httpConnections")]
    pub http_connections: usize,
    #[serde(rename = "masterNode")]
    pub master_node: Option<String>,
    #[serde(rename = "usedHttpConnections")]
    pub used_http_connections: i64,
}

impl StatusBarModel {
    pub fn new(
        app: &AppContext,
        tcp_connections: usize,
        http_connections: usize,
        tables_amount: usize,
        used_http_connections: i64,
    ) -> Self {
        Self {
            master_node: None,
            location: LocationModel {
                id: app.settings.location.to_string(),
                compress: app.settings.compress_data,
            },
            persist_amount: app.get_persist_amount(),
            tcp_connections,
            http_connections,
            tables_amount,
            used_http_connections,
        }
    }
}
