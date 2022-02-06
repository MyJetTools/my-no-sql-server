use my_json::json_writer::JsonArrayWriter;
use rust_extensions::date_time::{AtomicDateTimeAsMicroseconds, DateTimeAsMicroseconds};

use std::{
    collections::{btree_map::Values, BTreeMap, HashMap},
    sync::Arc,
};

use crate::{
    db::{
        db_snapshots::{DbPartitionSnapshot, DbRowsByPartitionsSnapshot},
        DbPartition, DbRow,
    },
    db_json_entity::JsonTimeStamp,
    persistence::DataToPersist,
    rows_with_expiration::RowsWithExpiration,
};

use super::DbTableDataIterator;

pub type TPartitions = BTreeMap<String, DbPartition>;

pub struct DbTableData {
    pub name: String,
    pub partitions: TPartitions,

    pub created: DateTimeAsMicroseconds,
    pub last_update_time: AtomicDateTimeAsMicroseconds,
    pub last_read_time: AtomicDateTimeAsMicroseconds,
    pub table_size: usize,
    pub rows_amount: usize,
    pub data_to_persist: DataToPersist,
    rows_with_expiration: RowsWithExpiration,
}

impl DbTableData {
    pub fn new(name: String, created: DateTimeAsMicroseconds) -> Self {
        Self {
            name,
            partitions: BTreeMap::new(),
            last_update_time: AtomicDateTimeAsMicroseconds::new(created.unix_microseconds),
            created,
            last_read_time: AtomicDateTimeAsMicroseconds::new(created.unix_microseconds),
            table_size: 0,
            rows_amount: 0,
            rows_with_expiration: RowsWithExpiration::new(),
            data_to_persist: DataToPersist::new(),
        }
    }

    pub fn get_partitions_amount(&self) -> usize {
        self.partitions.len()
    }

    pub fn iterate_all_rows<'s>(&'s self) -> DbTableDataIterator<'s> {
        DbTableDataIterator::new(&self.partitions)
    }

    pub fn get_table_as_json_array(&self) -> JsonArrayWriter {
        let mut json_array_writer = JsonArrayWriter::new();

        for db_partition in self.partitions.values() {
            for db_row in db_partition.rows.values() {
                json_array_writer.write_raw_element(db_row.data.as_slice())
            }
        }

        json_array_writer
    }

    pub fn get_partition_as_json_array(&self, partition_key: &str) -> Option<JsonArrayWriter> {
        let mut json_array_writer = JsonArrayWriter::new();

        if let Some(db_partition) = self.partitions.get(partition_key) {
            for db_row in db_partition.rows.values() {
                json_array_writer.write_raw_element(db_row.data.as_slice())
            }
        }

        json_array_writer.into()
    }

    #[inline]
    pub fn get_partition_mut(&mut self, partition_key: &str) -> Option<&mut DbPartition> {
        self.partitions.get_mut(partition_key)
    }

    #[inline]
    pub fn get_partition(&self, partition_key: &str) -> Option<&DbPartition> {
        self.partitions.get(partition_key)
    }
    #[inline]
    pub fn get_partitions(&self) -> Values<String, DbPartition> {
        self.partitions.values()
    }

    pub fn get_partitions_last_write_moment(&self) -> HashMap<String, DateTimeAsMicroseconds> {
        let mut result = HashMap::new();

        for (partition_key, db_partition) in &self.partitions {
            result.insert(
                partition_key.to_string(),
                db_partition.last_write_moment.as_date_time(),
            );
        }

        result
    }

    pub fn get_expired_rows_up_to(
        &mut self,
        now: DateTimeAsMicroseconds,
    ) -> Option<Vec<Arc<DbRow>>> {
        self.rows_with_expiration.remove_up_to(now)
    }
}

/// Insert Operations

impl DbTableData {
    #[inline]
    pub fn insert_or_replace_row(
        &mut self,
        db_row: &Arc<DbRow>,
        update_write_access: &JsonTimeStamp,
    ) -> Option<Arc<DbRow>> {
        if !self.partitions.contains_key(&db_row.partition_key) {
            self.handle_after_insert_row(&db_row);

            let mut db_partition = DbPartition::new();
            db_partition.insert(db_row.clone());

            self.partitions
                .insert(db_row.partition_key.to_string(), db_partition);

            return None;
        }

        let removed_row = {
            let db_partition = self.partitions.get_mut(&db_row.partition_key).unwrap();
            let removed_row = db_partition.insert(db_row.clone());
            db_partition
                .last_write_moment
                .update(update_write_access.date_time);

            if let Some(removed_row) = &removed_row {
                self.handle_after_remove_row(removed_row);
            }

            removed_row
        };

        self.handle_after_insert_row(db_row);

        return removed_row;
    }

    #[inline]
    pub fn insert_row(&mut self, db_row: &Arc<DbRow>, update_write_access: &JsonTimeStamp) -> bool {
        if !self.partitions.contains_key(&db_row.partition_key) {
            self.partitions
                .insert(db_row.partition_key.to_string(), DbPartition::new());
        }

        {
            let db_partition = self.partitions.get_mut(&db_row.partition_key).unwrap();

            if db_partition.rows.contains_key(&db_row.row_key) {
                return false;
            }

            db_partition.insert(db_row.clone());
            db_partition
                .last_write_moment
                .update(update_write_access.date_time);
        }

        self.handle_after_insert_row(&db_row);

        return true;
    }

    #[inline]
    pub fn bulk_insert_or_replace(
        &mut self,
        partition_key: &str,
        db_rows: &[Arc<DbRow>],
        update_write_access: &JsonTimeStamp,
    ) {
        if !self.partitions.contains_key(partition_key) {
            self.partitions
                .insert(partition_key.to_string(), DbPartition::new());
        }

        let removed_rows = {
            let db_partition = self.partitions.get_mut(partition_key).unwrap();
            db_partition
                .last_write_moment
                .update(update_write_access.date_time);

            let mut removed_rows = None;

            for db_row in db_rows {
                let removed_row = db_partition.insert(db_row.clone());
                if let Some(removed_row) = removed_row {
                    if removed_rows.is_none() {
                        removed_rows = Some(Vec::new());
                    }

                    removed_rows.as_mut().unwrap().push(removed_row);
                }
            }

            removed_rows
        };

        for db_row in db_rows {
            self.handle_after_insert_row(&db_row);
        }

        if let Some(removed_rows) = removed_rows {
            for removed_row in removed_rows {
                self.handle_after_remove_row(&removed_row);
            }
        }
    }

    #[inline]
    pub fn init_partition(&mut self, partition_key: String, db_partition: DbPartition) {
        for db_row in db_partition.rows.values() {
            self.handle_after_insert_row(db_row);
        }

        self.partitions.insert(partition_key, db_partition);
    }

    #[inline]
    pub fn handle_after_insert_row(&mut self, db_row: &Arc<DbRow>) {
        self.table_size += db_row.data.len();
        self.rows_amount += 1;
        self.rows_with_expiration.add(db_row);
    }
}

/// Delete Oprations
///
///

impl DbTableData {
    pub fn remove_row(
        &mut self,
        partition_key: &str,
        row_key: &str,
        delete_empty_partition: bool,
        now: &JsonTimeStamp,
    ) -> Option<(Arc<DbRow>, bool)> {
        let (removed_row, partition_is_empty) = {
            let db_partition = self.partitions.get_mut(partition_key)?;

            let removed_row = db_partition.rows.remove(row_key)?;

            db_partition.last_write_moment.update(now.date_time);

            (removed_row, db_partition.is_empty())
        };

        self.handle_after_remove_row(&removed_row);

        if delete_empty_partition && partition_is_empty {
            self.partitions.remove(partition_key);
        }

        return Some((removed_row, partition_is_empty));
    }

    pub fn bulk_remove_rows<'s, TIter: Iterator<Item = &'s str>>(
        &mut self,
        partition_key: &str,
        row_keys: TIter,
        delete_empty_partition: bool,
        now: &JsonTimeStamp,
    ) -> Option<(Vec<Arc<DbRow>>, bool)> {
        let (removed_rows, partition_is_empty) = {
            let db_partition = self.partitions.get_mut(partition_key)?;

            let mut result = None;

            for row_key in row_keys {
                let removed_row = db_partition.rows.remove(row_key);

                if let Some(removed_row) = removed_row {
                    if result.is_none() {
                        result = Some(Vec::new());
                    }

                    result.as_mut().unwrap().push(removed_row);
                }
            }

            if result.is_some() {
                db_partition.last_write_moment.update(now.date_time);
            }

            (result, db_partition.is_empty())
        };

        let removed_rows = removed_rows?;

        if delete_empty_partition && partition_is_empty {
            self.partitions.remove(partition_key);
        }

        for removed_row in &removed_rows {
            self.handle_after_remove_row(removed_row.as_ref());
        }

        return Some((removed_rows, partition_is_empty));
    }

    pub fn gc_and_keep_max_partitions_amount(
        &mut self,
        max_partitions_amount: usize,
    ) -> Option<HashMap<String, DbPartition>> {
        if self.partitions.len() <= max_partitions_amount {
            return None;
        }

        let mut partitions_to_gc = BTreeMap::new();

        for (partition_key, partition) in &self.partitions {
            let mut last_read_access = partition.get_last_access().unix_microseconds;

            while partitions_to_gc.contains_key(&last_read_access) {
                last_read_access += 1;
            }

            partitions_to_gc.insert(last_read_access, partition_key.to_string());
        }

        let mut result = HashMap::new();

        for (_, partition_key) in partitions_to_gc {
            if self.partitions.len() <= max_partitions_amount {
                break;
            }

            if let Some(partition) = self.partitions.remove(partition_key.as_str()) {
                result.insert(partition_key, partition);
            }
        }

        Some(result)
    }

    #[inline]
    pub fn remove_partition(&mut self, partition_key: &str) -> Option<DbPartition> {
        let removed_partition = self.partitions.remove(partition_key);
        if let Some(db_partition) = &removed_partition {
            self.handle_after_remove_partition(db_partition);
        }

        removed_partition
    }

    pub fn clean_table(&mut self) -> Option<TPartitions> {
        if self.partitions.len() == 0 {
            return None;
        }

        let mut partitions = BTreeMap::new();

        std::mem::swap(&mut partitions, &mut self.partitions);

        for db_partition in partitions.values() {
            self.handle_after_remove_partition(db_partition);
        }

        Some(partitions)
    }

    #[inline]
    fn handle_after_remove_partition(&mut self, db_partition: &DbPartition) {
        for db_row in db_partition.rows.values() {
            self.handle_after_remove_row(db_row.as_ref());
        }
    }

    #[inline]
    fn handle_after_remove_row(&mut self, db_row: &DbRow) {
        self.table_size -= db_row.data.len();
        self.rows_amount -= 1;
        self.rows_with_expiration.remove(db_row);
    }
}

impl Into<DbRowsByPartitionsSnapshot> for &DbTableData {
    fn into(self) -> DbRowsByPartitionsSnapshot {
        let mut result = DbRowsByPartitionsSnapshot::new();
        for (partition_key, db_partition) in &self.partitions {
            result.insert_partition(partition_key.to_string(), db_partition);
        }

        result
    }
}

impl Into<BTreeMap<String, DbPartitionSnapshot>> for &DbTableData {
    fn into(self) -> BTreeMap<String, DbPartitionSnapshot> {
        let mut result: BTreeMap<String, DbPartitionSnapshot> = BTreeMap::new();

        for (partition_key, db_partition) in &self.partitions {
            result.insert(partition_key.to_string(), db_partition.into());
        }

        result
    }
}
