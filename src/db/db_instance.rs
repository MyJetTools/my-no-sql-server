use tokio::sync::RwLock;

use super::{DbOperationFail, DbTable};
use std::{collections::HashMap, sync::Arc};
pub struct DbInstance {
    pub tables: RwLock<HashMap<String, Arc<DbTable>>>,
}

impl DbInstance {
    pub fn new() -> DbInstance {
        DbInstance {
            tables: RwLock::new(HashMap::new()),
        }
    }

    pub async fn get_table_names(&self) -> Vec<String> {
        let mut result = Vec::new();

        let read_access = self.tables.read().await;
        for table_name in read_access.keys() {
            result.push(table_name.to_string());
        }

        return result;
    }

    pub async fn get_table(&self, table_name: &str) -> Result<Arc<DbTable>, DbOperationFail> {
        let read_access = self.tables.read().await;

        let result = read_access.get(table_name);

        return match result {
            Some(table) => Ok(table.clone()),
            None => Err(DbOperationFail::TableNotFound {
                table_name: table_name.to_string(),
            }),
        };
    }
}
