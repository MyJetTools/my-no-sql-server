use crate::date_time::{AtomicDateTime, MyDateTime};
use crate::utils::SortedHashMap;
use std::{collections::BTreeMap, sync::Arc};

use super::DbRow;

pub struct DbPartition {
    pub rows: BTreeMap<String, Arc<DbRow>>,
    pub last_access: AtomicDateTime,
}

impl DbPartition {
    pub fn new() -> DbPartition {
        DbPartition {
            rows: BTreeMap::new(),
            last_access: AtomicDateTime::utc_now(),
        }
    }

    pub fn update_last_access(&self, now: MyDateTime) {
        self.last_access.update(now);
    }

    /*
       pub fn as_bytes(&self) -> Vec<u8> {
           let mut json = Vec::new();

           for db_row in self.rows.values() {
               if json.len() == 0 {
                   json.push(crate::json::consts::OPEN_ARRAY);
               } else {
                   json.push(crate::json::consts::COMMA);
               }

               json.extend(db_row.data.as_slice());
           }

           json.push(crate::json::consts::CLOSE_ARRAY);

           return json;
       }
    */

    pub fn insert(&mut self, db_row: Arc<DbRow>) -> bool {
        if self.rows.contains_key(db_row.row_key.as_str()) {
            return false;
        }

        self.rows.insert(db_row.row_key.to_string(), db_row);

        return true;
    }

    pub fn insert_or_replace(&mut self, db_row: Arc<DbRow>) {
        self.rows.insert(db_row.row_key.to_string(), db_row);
    }

    pub fn get_rows_amount(&self) -> usize {
        return self.rows.len();
    }

    pub fn gc_rows(&mut self, max_rows_amount: usize) -> Vec<Arc<DbRow>> {
        let mut partitions_by_date_time: SortedHashMap<i64, String> = SortedHashMap::new();

        let mut gced = Vec::new();

        for (row_key, db_partition) in &self.rows {
            let mut last_access = db_partition.last_access.get();
            let last_access_before_insert = last_access;

            while partitions_by_date_time.contains_key(&last_access) {
                last_access += 1;
            }

            partitions_by_date_time.insert(last_access, row_key.to_string());

            if last_access_before_insert != last_access {
                db_partition.last_access.update_value(last_access);
            }
        }

        while self.rows.len() > max_rows_amount {
            let (dt, partition_key) = partitions_by_date_time.first().unwrap();

            let removed_result = self.rows.remove(&partition_key);

            if let Some(el) = removed_result {
                gced.push(el);
            }

            partitions_by_date_time.remove(&dt);
        }

        gced
    }

    pub fn get_row_and_update_last_time(
        &self,
        row_key: &str,
        now: MyDateTime,
    ) -> Option<Arc<DbRow>> {
        let result = self.rows.get(row_key)?;
        result.update_last_access(now);
        return Some(result.clone());
    }

    pub fn get_highest_row_and_below(&self, row_key: String, now: MyDateTime) -> Vec<Arc<DbRow>> {
        let mut result = Vec::new();

        for (_, db_row) in self.rows.range(..row_key) {
            db_row.update_last_access(now);
            result.push(db_row.clone());
        }

        result
    }
}
