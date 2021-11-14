use std::{collections::BTreeMap, sync::Arc};

use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::db::DbRow;

use super::RowsWithExpirationBucket;

type ItemsWithExpiration = BTreeMap<i64, RowsWithExpirationBucket>;

pub struct RowsWithExpiration {
    items: ItemsWithExpiration,
}

impl RowsWithExpiration {
    pub fn new() -> Self {
        Self {
            items: BTreeMap::new(),
        }
    }

    pub fn add(&mut self, db_row: &Arc<DbRow>) {
        if let Some(expires) = db_row.get_expires() {
            if self.items.contains_key(&expires.unix_microseconds) {
                self.items
                    .get_mut(&expires.unix_microseconds)
                    .unwrap()
                    .add(db_row.clone());
            } else {
                self.items.insert(
                    expires.unix_microseconds,
                    RowsWithExpirationBucket::new(db_row.clone()),
                );
            }
        }
    }

    pub fn remove(&mut self, db_row: &DbRow) {
        let db_row_expires = db_row.get_expires();
        if let Some(expires) = db_row_expires {
            let mut bucket_is_empty = false;

            let bucket = self.items.get_mut(&expires.unix_microseconds);

            if let Some(bucket) = bucket {
                bucket.remove(db_row);
                bucket_is_empty = bucket.is_empty();
            };

            if bucket_is_empty {
                self.items.remove(&expires.unix_microseconds);
            }
        }
    }

    pub fn remove_up_to(&mut self, now: DateTimeAsMicroseconds) -> Option<Vec<Arc<DbRow>>> {
        let items_to_remove = get_keys_to_remove(&mut self.items, now)?;

        let mut result = None;
        for item_to_remove in items_to_remove {
            let bucket = self.items.remove(&item_to_remove);

            if let Some(bucket) = bucket {
                if result.is_none() {
                    result = Some(bucket.db_rows)
                } else {
                    result.as_mut().unwrap().extend(bucket.db_rows);
                }
            }
        }

        result
    }
}

#[inline]
fn get_keys_to_remove(
    items: &ItemsWithExpiration,
    now: DateTimeAsMicroseconds,
) -> Option<Vec<i64>> {
    let mut result = None;

    for (time_stamp, _) in items {
        if now.unix_microseconds < *time_stamp {
            break;
        }

        if result.is_none() {
            result = Some(Vec::new());
        }

        result.as_mut().unwrap().push(*time_stamp);
    }

    return result;
}
