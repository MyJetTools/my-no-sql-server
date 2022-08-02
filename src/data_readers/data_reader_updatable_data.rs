use std::{collections::HashMap, sync::Arc};

use crate::db::DbTableWrapper;

pub struct DataReaderUpdatableData {
    tables: HashMap<String, Arc<DbTableWrapper>>,
}

impl DataReaderUpdatableData {
    pub fn new() -> Self {
        Self {
            tables: HashMap::new(),
        }
    }

    pub fn subscribe(&mut self, db_table_wrapper: Arc<DbTableWrapper>) {
        self.tables
            .insert(db_table_wrapper.name.to_string(), db_table_wrapper);
    }

    pub fn unsubscribe(&mut self, table_name: &str) {
        self.tables.remove(table_name);
    }

    pub fn has_table(&self, table_name: &str) -> bool {
        self.tables.contains_key(table_name)
    }

    pub fn get_table_names(&self) -> Vec<String> {
        self.tables.keys().map(|id| id.to_string()).collect()
    }
}
