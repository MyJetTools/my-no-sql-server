use serde::{Deserialize, Serialize};

/// One entry of `GET /api/Namespaces/List`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct NamespaceApiModel {
    pub name: String,
    #[serde(rename = "tablesAmount")]
    pub tables_amount: usize,
}
