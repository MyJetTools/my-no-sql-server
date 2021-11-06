use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};

use rust_extensions::date_time::DateTimeAsMicroseconds;
use tokio::sync::Mutex;

use crate::db::DbRow;

use super::RowWithExpirationBucket;

pub struct RowsWithExpiration {
    items: Mutex<BTreeMap<i64, HashMap<String, RowWithExpirationBucket>>>,
}

impl RowsWithExpiration {
    pub fn new() -> Self {
        Self {
            items: Mutex::new(BTreeMap::new()),
        }
    }

    pub async fn add(&self, table_name: &str, db_row: Arc<DbRow>) {
        if let Some(expires) = db_row.get_expires() {
            let mut write_access = self.items.lock().await;
            add_internal(
                &mut write_access,
                table_name,
                db_row,
                expires.unix_microseconds,
            );
        }
    }

    pub async fn add_multiple(&self, table_name: &str, db_rows: &[Arc<DbRow>]) {
        let mut write_access = self.items.lock().await;
        for db_row in db_rows {
            let db_row_expires = db_row.get_expires();
            if let Some(expires) = db_row_expires {
                add_internal(
                    &mut write_access,
                    table_name,
                    db_row.clone(),
                    expires.unix_microseconds,
                );
            }
        }
    }

    pub async fn removed(&self, table_name: &str, db_row: &DbRow) {
        let db_row_expires = db_row.get_expires();
        if let Some(expires) = db_row_expires {
            let mut write_access = self.items.lock().await;
            remove_internal(&mut write_access, expires, table_name, db_row);
        }
    }

    pub async fn bulk_removed<'s, TIter: Iterator<Item = &'s Arc<DbRow>>>(
        &self,
        table_name: &str,
        db_rows: TIter,
    ) {
        let mut write_access = self.items.lock().await;

        for db_row in db_rows {
            let db_row_expires = db_row.get_expires();
            if let Some(expires) = db_row_expires {
                remove_internal(&mut write_access, expires, table_name, db_row);
            }
        }
    }
}

fn add_internal(
    items: &mut BTreeMap<i64, HashMap<String, RowWithExpirationBucket>>,
    table_name: &str,
    db_row: Arc<DbRow>,
    expires: i64,
) {
    if items.contains_key(&expires) {
        let by_table = items.get_mut(&expires).unwrap();

        if by_table.contains_key(table_name) {
            by_table.get_mut(table_name).unwrap().add(db_row);
        } else {
            by_table.insert(table_name.to_string(), RowWithExpirationBucket::new(db_row));
        }
    } else {
        let mut by_table = HashMap::new();
        by_table.insert(table_name.to_string(), RowWithExpirationBucket::new(db_row));
        items.insert(expires, by_table);
    }
}

fn remove_internal(
    items: &mut BTreeMap<i64, HashMap<String, RowWithExpirationBucket>>,
    expires: DateTimeAsMicroseconds,
    table_name: &str,
    db_row: &DbRow,
) {
    let mut bucket_is_empty = false;

    let mut slot_is_empty = false;

    {
        let by_table = items.get_mut(&expires.unix_microseconds);

        if by_table.is_none() {
            return;
        }

        let by_table = by_table.unwrap();
        {
            let bucket = by_table.get_mut(table_name);

            if let Some(bucket) = bucket {
                bucket.remove(db_row);

                bucket_is_empty = bucket.is_empty();
            };
        }

        if bucket_is_empty {
            by_table.remove(table_name);

            slot_is_empty = by_table.len() == 0;
        }
    }

    if slot_is_empty {
        items.remove(&expires.unix_microseconds);
    }
}
