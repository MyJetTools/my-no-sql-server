use std::{
    collections::{
        btree_map::{Range, Values},
        BTreeMap, HashMap,
    },
    ops::RangeBounds,
    sync::Arc,
};

use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{db::DbRow, utils::LazyVec};

pub struct DbRowsContainer {
    data: BTreeMap<String, Arc<DbRow>>,
    rows_with_expiration_index: BTreeMap<i64, HashMap<String, Arc<DbRow>>>,

    content_size: usize,
}

impl DbRowsContainer {
    pub fn new() -> Self {
        Self {
            data: BTreeMap::new(),
            rows_with_expiration_index: BTreeMap::new(),
            content_size: 0,
        }
    }

    pub fn get_content_size(&self) -> usize {
        self.content_size
    }

    fn insert_to_data(&mut self, db_row: Arc<DbRow>) -> Option<Arc<DbRow>> {
        self.content_size += db_row.data.len();
        let result = self.data.insert(db_row.row_key.to_string(), db_row);

        if let Some(removed_item) = &result {
            self.content_size -= removed_item.data.len();
        }

        result
    }

    fn remove_from_data(&mut self, row_key: &str) -> Option<Arc<DbRow>> {
        let result = self.data.remove(row_key);

        if let Some(removed_item) = &result {
            self.content_size -= removed_item.data.len();
        }

        result
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn has_rows_to_expire(&self, now: DateTimeAsMicroseconds) -> bool {
        for dt in self.rows_with_expiration_index.keys() {
            return *dt <= now.unix_microseconds;
        }

        false
    }

    pub fn expire_rows(&mut self, now: DateTimeAsMicroseconds) -> Option<Vec<Arc<DbRow>>> {
        let mut keys = LazyVec::new();

        for dt in self.rows_with_expiration_index.keys() {
            if *dt <= now.unix_microseconds {
                keys.push(*dt);
            }
        }

        let keys = keys.get_result()?;

        let mut result = Vec::new();

        for key in &keys {
            if let Some(items) = self.rows_with_expiration_index.remove(key) {
                for (row_key, db_row) in items {
                    self.remove_from_data(row_key.as_str());
                    result.push(db_row);
                }
            }
        }

        Some(result)
    }

    fn insert_expiration_index(&mut self, db_row: &Arc<DbRow>) {
        let expires = db_row.get_expires();
        if expires.is_none() {
            return;
        }

        let expires = expires.unwrap();

        if !self
            .rows_with_expiration_index
            .contains_key(&expires.unix_microseconds)
        {
            self.rows_with_expiration_index
                .insert(expires.unix_microseconds, HashMap::new());
        }

        if let Some(index_data) = self
            .rows_with_expiration_index
            .get_mut(&expires.unix_microseconds)
        {
            index_data.insert(db_row.row_key.to_string(), db_row.clone());
        }
    }

    fn remove_expiration_index(&mut self, db_row: &Arc<DbRow>) {
        let expires = db_row.get_expires();
        if expires.is_none() {
            return;
        }

        let expires = expires.unwrap();

        let remove_root_index = if let Some(index_data) = self
            .rows_with_expiration_index
            .get_mut(&expires.unix_microseconds)
        {
            index_data.remove(&db_row.row_key);
            index_data.len() == 0
        } else {
            false
        };

        if remove_root_index {
            self.rows_with_expiration_index
                .remove(&expires.unix_microseconds);
        }
    }

    fn insert_indices(&mut self, db_row: &Arc<DbRow>) {
        self.insert_expiration_index(&db_row);
    }

    fn remove_indices(&mut self, db_row: &Arc<DbRow>) {
        self.remove_expiration_index(&db_row);
    }

    pub fn insert(&mut self, db_row: Arc<DbRow>) -> Option<Arc<DbRow>> {
        self.insert_indices(&db_row);
        let result = self.insert_to_data(db_row);

        if let Some(removed_db_row) = &result {
            self.remove_indices(&removed_db_row);
        }

        result
    }

    pub fn remove(&mut self, row_key: &str) -> Option<Arc<DbRow>> {
        let result = self.remove_from_data(row_key);

        if let Some(removed_db_row) = &result {
            self.remove_indices(&removed_db_row);
        }

        result
    }

    pub fn get(&self, row_key: &str) -> Option<&Arc<DbRow>> {
        self.data.get(row_key)
    }

    pub fn has_db_row(&self, row_key: &str) -> bool {
        return self.data.contains_key(row_key);
    }

    pub fn get_all<'s>(&'s self) -> Values<'s, String, Arc<DbRow>> {
        self.data.values()
    }

    pub fn range<'s, R>(&'s self, range: R) -> Range<'s, String, Arc<DbRow>>
    where
        R: RangeBounds<String>,
    {
        self.data.range(range)
    }

    fn get_db_rows(&self, row_keys: &[String]) -> Option<Vec<Arc<DbRow>>> {
        let mut result = None;
        for row_key in row_keys {
            if let Some(db_row) = self.data.get(row_key) {
                if result.is_none() {
                    result = Some(Vec::new());
                }

                result.as_mut().unwrap().push(db_row.clone());
            }
        }

        result
    }

    pub fn update_expiration_time(
        &mut self,
        row_keys: &[String],
        expiration_time: Option<DateTimeAsMicroseconds>,
    ) {
        let db_rows = self.get_db_rows(row_keys);

        if db_rows.is_none() {
            return;
        }

        let db_rows = db_rows.unwrap();

        for db_row in db_rows {
            self.remove_expiration_index(&db_row);
            db_row.update_expires(expiration_time);
            if expiration_time.is_some() {
                self.insert_expiration_index(&db_row);
            }
        }
    }

    pub fn gc_rows(&mut self, now: DateTimeAsMicroseconds) -> Option<Vec<Arc<DbRow>>> {
        let mut result = LazyVec::new();

        for (expires, items) in &self.rows_with_expiration_index {
            if now.unix_microseconds < *expires {
                break;
            }

            for item in items.values() {
                result.push(item.clone());
            }
        }

        let result = result.get_result();

        if let Some(gced_db_rows) = result.as_ref() {
            for gced_db_row in gced_db_rows {
                self.remove(&gced_db_row.row_key);
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {

    use rust_extensions::date_time::DateTimeAsMicroseconds;

    use crate::db_json_entity::JsonTimeStamp;

    use super::*;

    #[test]
    fn test_that_index_appears() {
        let mut db_rows = DbRowsContainer::new();

        let db_row = DbRow::new(
            "testPartitionKey".to_string(),
            "testRowKey".to_string(),
            vec![],
            Some(DateTimeAsMicroseconds::new(1)),
            &JsonTimeStamp::now(),
        );

        db_rows.insert(Arc::new(db_row));

        assert_eq!(1, db_rows.rows_with_expiration_index.len())
    }

    #[test]
    fn test_that_index_does_not_appear_since_we_do_not_have_expiration() {
        let mut db_rows = DbRowsContainer::new();

        let db_row = DbRow::new(
            "testPartitionKey".to_string(),
            "testRowKey".to_string(),
            vec![],
            None,
            &JsonTimeStamp::now(),
        );

        db_rows.insert(Arc::new(db_row));

        assert_eq!(0, db_rows.rows_with_expiration_index.len())
    }

    #[test]
    fn test_that_index_dissapears() {
        let mut db_rows = DbRowsContainer::new();

        let db_row = DbRow::new(
            "testPartitionKey".to_string(),
            "testRowKey".to_string(),
            vec![],
            Some(DateTimeAsMicroseconds::new(1)),
            &JsonTimeStamp::now(),
        );

        let db_row = Arc::new(db_row);
        db_rows.insert(db_row.clone());

        db_rows.remove(db_row.row_key.as_str());

        assert_eq!(0, db_rows.rows_with_expiration_index.len())
    }

    #[test]
    fn test_update_expiration_time_from_no_to() {
        const ROW_KEY: &str = "testRowKey";
        let mut db_rows = DbRowsContainer::new();

        let db_row = DbRow::new(
            "testPartitionKey".to_string(),
            ROW_KEY.to_string(),
            vec![],
            None,
            &JsonTimeStamp::now(),
        );

        let db_row = Arc::new(db_row);
        db_rows.insert(db_row.clone());

        assert_eq!(0, db_rows.rows_with_expiration_index.len());

        db_rows.update_expiration_time(
            &vec![ROW_KEY.to_string()],
            Some(DateTimeAsMicroseconds::new(2)),
        );

        assert_eq!(true, db_rows.rows_with_expiration_index.contains_key(&2));
        assert_eq!(1, db_rows.rows_with_expiration_index.len());
    }

    #[test]
    fn test_update_expiration_time_to_new_expiration_time() {
        const ROW_KEY: &str = "testRowKey";
        let mut db_rows = DbRowsContainer::new();

        let db_row = DbRow::new(
            "testPartitionKey".to_string(),
            ROW_KEY.to_string(),
            vec![],
            Some(DateTimeAsMicroseconds::new(1)),
            &JsonTimeStamp::now(),
        );

        let db_row = Arc::new(db_row);
        db_rows.insert(db_row.clone());

        assert_eq!(true, db_rows.rows_with_expiration_index.contains_key(&1));
        assert_eq!(1, db_rows.rows_with_expiration_index.len());

        db_rows.update_expiration_time(
            &vec![ROW_KEY.to_string()],
            Some(DateTimeAsMicroseconds::new(2)),
        );

        assert_eq!(true, db_rows.rows_with_expiration_index.contains_key(&2));
        assert_eq!(1, db_rows.rows_with_expiration_index.len());
    }

    #[test]
    fn test_update_expiration_time_from_some_to_no() {
        const ROW_KEY: &str = "testRowKey";
        let mut db_rows = DbRowsContainer::new();

        let db_row = DbRow::new(
            "testPartitionKey".to_string(),
            ROW_KEY.to_string(),
            vec![],
            Some(DateTimeAsMicroseconds::new(1)),
            &JsonTimeStamp::now(),
        );

        let db_row = Arc::new(db_row);
        db_rows.insert(db_row.clone());

        assert_eq!(true, db_rows.rows_with_expiration_index.contains_key(&1));
        assert_eq!(1, db_rows.rows_with_expiration_index.len());

        db_rows.update_expiration_time(&vec![ROW_KEY.to_string()], None);

        assert_eq!(0, db_rows.rows_with_expiration_index.len());
    }

    #[test]
    fn test_we_do_not_have_db_rows_to_expire() {
        const ROW_KEY: &str = "testRowKey";
        let mut db_rows = DbRowsContainer::new();

        let db_row = DbRow::new(
            "testPartitionKey".to_string(),
            ROW_KEY.to_string(),
            vec![],
            Some(DateTimeAsMicroseconds::new(10)),
            &JsonTimeStamp::now(),
        );

        let db_row = Arc::new(db_row);
        db_rows.insert(db_row.clone());

        assert_eq!(
            false,
            db_rows.has_rows_to_expire(DateTimeAsMicroseconds::new(9))
        );
    }

    #[test]
    fn test_we_do_have_db_rows_to_expire() {
        const ROW_KEY: &str = "testRowKey";
        let mut db_rows = DbRowsContainer::new();

        let db_row = DbRow::new(
            "testPartitionKey".to_string(),
            ROW_KEY.to_string(),
            vec![],
            Some(DateTimeAsMicroseconds::new(10)),
            &JsonTimeStamp::now(),
        );

        let db_row = Arc::new(db_row);
        db_rows.insert(db_row.clone());

        assert_eq!(
            true,
            db_rows.has_rows_to_expire(DateTimeAsMicroseconds::new(10))
        );
    }

    #[test]
    fn test_db_rows_expiration() {
        let mut db_rows = DbRowsContainer::new();

        let db_row = DbRow::new(
            "testPartitionKey".to_string(),
            "testRowKey".to_string(),
            vec![],
            Some(DateTimeAsMicroseconds::new(10)),
            &JsonTimeStamp::now(),
        );

        let db_row = Arc::new(db_row);
        db_rows.insert(db_row.clone());

        let db_row = DbRow::new(
            "testPartitionKey".to_string(),
            "testRowKey1".to_string(),
            vec![],
            Some(DateTimeAsMicroseconds::new(11)),
            &JsonTimeStamp::now(),
        );

        let db_row = Arc::new(db_row);
        db_rows.insert(db_row.clone());

        let expired = db_rows
            .expire_rows(DateTimeAsMicroseconds::new(10))
            .unwrap();

        assert_eq!(1, expired.len());

        assert_eq!(1, db_rows.data.len());
        assert_eq!(1, db_rows.rows_with_expiration_index.len());

        assert_eq!("testRowKey", expired[0].row_key);
    }
}
