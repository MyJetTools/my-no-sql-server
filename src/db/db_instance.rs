use tokio::sync::RwLock;

use super::DbTable;
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
        let read_access = self.tables.read().await;

        return read_access
            .values()
            .into_iter()
            .map(|table| table.name.clone())
            .collect();
    }

    pub async fn get_tables(&self) -> Vec<Arc<DbTable>> {
        let read_access = self.tables.read().await;

        return read_access
            .values()
            .into_iter()
            .map(|table| table.clone())
            .collect();
    }

    pub async fn get_tables_with_common_persist_thread(&self) -> Vec<Arc<DbTable>> {
        let read_access = self.tables.read().await;

        let mut result = Vec::new();

        for db_table in read_access.values() {
            if db_table.persist_using_common_thread() {}
            result.push(db_table.clone());
        }

        result
    }

    pub async fn get_table(&self, table_name: &str) -> Option<Arc<DbTable>> {
        let read_access = self.tables.read().await;

        let result = read_access.get(table_name)?;
        return Some(result.clone());
    }

    pub async fn delete_table(&self, table_name: &str) -> Option<Arc<DbTable>> {
        let mut write_access = self.tables.write().await;
        return write_access.remove(table_name);
    }
}
