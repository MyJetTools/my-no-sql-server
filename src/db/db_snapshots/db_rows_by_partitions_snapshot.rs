use std::{collections::BTreeMap, sync::Arc};

use my_json::json_writer::JsonArrayWriter;

use crate::db::{DbPartition, DbRow};

pub struct DbRowsByPartitionsSnapshot {
    pub partitions: BTreeMap<String, Option<Vec<Arc<DbRow>>>>,
}

impl DbRowsByPartitionsSnapshot {
    pub fn new() -> Self {
        Self {
            partitions: BTreeMap::new(),
        }
    }

    pub fn add_row(&mut self, db_row: Arc<DbRow>) {
        if !self.partitions.contains_key(&db_row.partition_key) {
            self.partitions
                .insert(db_row.partition_key.to_string(), Some(Vec::new()));
        }

        let partitions = self
            .partitions
            .get_mut(&db_row.partition_key)
            .unwrap()
            .as_mut()
            .unwrap();

        partitions.push(db_row);
    }

    pub fn add_rows(&mut self, partition_key: &str, db_rows: Vec<Arc<DbRow>>) {
        if !self.partitions.contains_key(partition_key) {
            self.partitions
                .insert(partition_key.to_string(), Some(Vec::new()));
        }

        let vec_to_add = self
            .partitions
            .get_mut(partition_key)
            .unwrap()
            .as_mut()
            .unwrap();

        for db_row in db_rows {
            vec_to_add.push(db_row);
        }
    }

    pub fn insert_partition(&mut self, partition_key: String, db_partition: &DbPartition) {
        let mut data_to_insert = None;

        for db_row in db_partition.rows.values() {
            if data_to_insert.is_none() {
                data_to_insert = Some(Vec::new());
            }

            data_to_insert.as_mut().unwrap().push(db_row.clone());
        }

        self.partitions.insert(partition_key, data_to_insert);
    }

    pub fn insert_empty_partition(&mut self, partition_key: String) {
        self.partitions.insert(partition_key, None);
    }

    pub fn as_json_array(&self) -> JsonArrayWriter {
        let mut result = JsonArrayWriter::new();
        for snapshot in self.partitions.values() {
            if let Some(snapshot) = snapshot {
                for db_row in snapshot {
                    result.write_raw_element(&db_row.data);
                }
            } else {
                result.write_null_element()
            }
        }

        result
    }
}
