use std::{collections::HashMap, sync::Arc};

use crate::db::DbTable;

pub struct DataReaderUpdatableData {
    tables: HashMap<String, Arc<DbTable>>,
    pub name: Option<String>,
}

impl DataReaderUpdatableData {
    pub fn new() -> Self {
        Self {
            tables: HashMap::new(),
            name: None,
        }
    }

    pub fn subscribe(&mut self, db_table: Arc<DbTable>) {
        self.tables.insert(db_table.name.to_string(), db_table);
    }

    pub fn has_table(&self, table_name: &str) -> bool {
        self.tables.contains_key(table_name)
    }

    pub fn update_name(&mut self, name: String) {
        self.name = Some(name);
    }

    pub fn get_table_names(&self) -> Vec<String> {
        self.tables.keys().map(|id| id.to_string()).collect()
    }
}
