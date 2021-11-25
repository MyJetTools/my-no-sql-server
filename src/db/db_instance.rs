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
        let mut result = Vec::new();

        let read_access = self.tables.read().await;
        for table_name in read_access.keys() {
            result.push(table_name.to_string());
        }

        return result;
    }

    pub async fn get_tables(&self) -> Vec<Arc<DbTable>> {
        let read_access = self.tables.read().await;

        return read_access
            .values()
            .into_iter()
            .map(|table| table.clone())
            .collect();
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

    pub async fn get_tables_which_partitions_restrictions(&self) -> Option<Vec<Arc<DbTable>>> {
        let mut result = None;
        let read_access = self.tables.read().await;

        for table in read_access.values() {
            if table.attributes.get_max_partitions_amount().is_some() {
                if result.is_none() {
                    result = Some(Vec::new());
                }

                result.as_mut().unwrap().push(table.clone());
            }
        }

        result
    }

    /*
    pub async fn get_or_create_table(
        &self,
        name: &str,
        persist: bool,
        max_partitions_amount: Option<usize>,
        now: DateTimeAsMicroseconds,
    ) -> CreateTableResult {
        let mut write_access = self.tables.write().await;

        if let Some(table) = write_access.get(name) {
            return CreateTableResult::AlreadyHadTable(table.clone());
        }

        let table_attributes = DbTableAttributes {
            persist,
            max_partitions_amount,
            created: now,
        };

        let db_table_data = DbTableData::new(name.to_string(), table_attributes);

        let new_table = DbTable::new(db_table_data, DateTimeAsMicroseconds::now());

        let new_table = Arc::new(new_table);
        write_access.insert(name.to_string(), new_table.clone());

        return CreateTableResult::JustCreated(new_table);
    }
     */
}
