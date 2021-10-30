use rust_extensions::date_time::{AtomicDateTimeAsMicroseconds, DateTimeAsMicroseconds};

use crate::{db::DbRow, json::JsonArrayBuilder, utils::SortedDictionary};
use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};

use super::DbPartitionSnapshot;

pub struct DbPartition {
    rows: BTreeMap<String, Arc<DbRow>>,
    pub last_read_access: AtomicDateTimeAsMicroseconds,
    pub last_write_access: AtomicDateTimeAsMicroseconds,
}

impl DbPartition {
    pub fn new() -> DbPartition {
        DbPartition {
            rows: BTreeMap::new(),
            last_read_access: AtomicDateTimeAsMicroseconds::now(),
            last_write_access: AtomicDateTimeAsMicroseconds::now(),
        }
    }

    pub fn clear(&mut self) -> bool {
        if self.rows.len() == 0 {
            return false;
        }

        self.rows.clear();
        return true;
    }

    pub fn rows_count(&self) -> usize {
        return self.rows.len();
    }

    pub fn update_last_access(&self, now: DateTimeAsMicroseconds) {
        self.last_read_access.update(now);
    }

    pub fn insert(
        &mut self,
        db_row: Arc<DbRow>,
        update_write_access: Option<DateTimeAsMicroseconds>,
    ) -> bool {
        if self.rows.contains_key(db_row.row_key.as_str()) {
            return false;
        }

        self.rows.insert(db_row.row_key.to_string(), db_row);

        if let Some(write_access_time) = update_write_access {
            self.last_write_access.update(write_access_time);
        }

        return true;
    }

    pub fn bulk_insert_or_replace(
        &mut self,
        db_rows: &[Arc<DbRow>],
        update_write_access: Option<DateTimeAsMicroseconds>,
    ) {
        for db_row in db_rows {
            self.rows.insert(db_row.row_key.to_string(), db_row.clone());
        }

        if let Some(write_access_time) = update_write_access {
            self.last_write_access.update(write_access_time);
        }
    }

    pub fn insert_or_replace(
        &mut self,
        db_row: Arc<DbRow>,
        update_write_access: Option<DateTimeAsMicroseconds>,
    ) {
        self.rows.insert(db_row.row_key.to_string(), db_row);

        if let Some(write_access_time) = update_write_access {
            self.last_write_access.update(write_access_time);
        }
    }

    pub fn remove(&mut self, row_key: &str, now: DateTimeAsMicroseconds) -> Option<Arc<DbRow>> {
        let result = self.rows.remove(row_key);

        if result.is_some() {
            self.last_write_access.update(now);
        }

        return result;
    }

    pub fn get_row(&self, row_key: &str) -> Option<&DbRow> {
        let result = self.rows.get(row_key)?;
        Some(result.as_ref())
    }

    pub fn get_and_clone_row(&self, row_key: &str) -> Option<Arc<DbRow>> {
        let result = self.rows.get(row_key)?;
        Some(result.clone())
    }

    pub fn get_rows_amount(&self) -> usize {
        return self.rows.len();
    }

    pub fn gc_rows(&mut self, max_rows_amount: usize) -> Option<HashMap<String, Arc<DbRow>>> {
        if self.rows.len() == 0 {
            return None;
        }

        let mut partitions_by_date_time: SortedDictionary<i64, String> = SortedDictionary::new();

        for (row_key, db_row) in &mut self.rows {
            let mut last_access = db_row.last_read_access.as_date_time();

            let last_access_before_insert = last_access;

            while partitions_by_date_time.contains_key(&last_access.unix_microseconds) {
                last_access.unix_microseconds += 1;
            }

            partitions_by_date_time.insert(last_access.unix_microseconds, row_key.to_string());

            if last_access_before_insert.unix_microseconds != last_access.unix_microseconds {
                db_row.last_read_access.update(last_access);
            }
        }

        let mut gced = None;

        while self.rows.len() > max_rows_amount {
            let (dt, row_key) = partitions_by_date_time.first().unwrap();

            let removed_result = self.rows.remove(&row_key);

            if let Some(db_row) = removed_result {
                if gced.is_none() {
                    gced = Some(HashMap::new())
                }

                gced.as_mut().unwrap().insert(row_key, db_row);
            }

            partitions_by_date_time.remove(&dt);
        }

        gced
    }

    pub fn get_highest_row_and_below(&self, row_key: &String) -> Vec<Arc<DbRow>> {
        let mut result = Vec::new();
        for (db_row_key, db_row) in self.rows.range(..row_key.to_string()) {
            if db_row_key <= row_key {
                result.push(db_row.clone());
            }
        }

        result
    }

    pub fn fill_with_json_data(
        &self,
        builder: &mut JsonArrayBuilder,
        update_read_access_time: Option<DateTimeAsMicroseconds>,
    ) {
        for db_row in self.rows.values() {
            if let Some(update_time) = update_read_access_time {
                db_row.last_read_access.update(update_time)
            }

            builder.append_json_object(db_row.data.as_slice());
        }
    }

    pub fn get_db_partition_snapshot(
        &self,
        update_read_access_time: Option<DateTimeAsMicroseconds>,
    ) -> DbPartitionSnapshot {
        DbPartitionSnapshot {
            last_read_access: self.last_read_access.as_date_time(),
            last_write_access: self.last_write_access.as_date_time(),
            content: self.rows.values().map(|itm| itm.clone()).collect(),
        }
    }
}
