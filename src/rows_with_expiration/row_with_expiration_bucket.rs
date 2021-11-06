use std::sync::Arc;

use crate::db::DbRow;

pub struct RowWithExpirationBucket {
    pub db_rows: Vec<Arc<DbRow>>,
}

impl RowWithExpirationBucket {
    pub fn new(db_row: Arc<DbRow>) -> Self {
        Self {
            db_rows: vec![db_row],
        }
    }

    pub fn add(&mut self, db_row: Arc<DbRow>) {
        self.db_rows.push(db_row);
    }

    fn get_index(&self, partition_key: &str, row_key: &str) -> Option<usize> {
        for index in 0..self.db_rows.len() {
            let item = self.db_rows.get(index).unwrap();

            if item.partition_key == partition_key && item.row_key == row_key {
                return Some(index);
            }
        }

        None
    }

    pub fn remove(&mut self, db_row: &DbRow) {
        let index = self.get_index(db_row.partition_key.as_str(), db_row.row_key.as_str());

        if let Some(index) = index {
            self.db_rows.remove(index);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.db_rows.len() == 0
    }
}
