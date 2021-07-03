use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};

use crate::utils::{date_time::MyDateTime, SortedHashMap};

use super::{DbPartition, DbRow, FailOperationResult, OperationResult};

#[derive(Debug, Clone)]
pub struct DbTableAttributes {
    pub persist: bool,
    pub max_partitions_amount: Option<usize>,
}

pub struct DbTableData {
    pub partitions: BTreeMap<String, DbPartition>,
    pub attributes: DbTableAttributes,
}

impl DbTableData {
    pub fn new(attributes: DbTableAttributes) -> Self {
        Self {
            partitions: BTreeMap::new(),
            attributes,
        }
    }

    pub fn init_partition(&mut self, partition_key: String, partition: DbPartition) {
        self.partitions.insert(partition_key, partition);
    }

    pub fn get_partition_and_update_last_access_mut(
        &mut self,
        partition_key: &str,
        now: MyDateTime,
    ) -> Option<&mut DbPartition> {
        let result = self.partitions.get_mut(partition_key)?;

        result.update_last_access(now);

        return Some(result);
    }

    pub fn get_partition_mut(&mut self, partition_key: &str) -> Option<&mut DbPartition> {
        return self.partitions.get_mut(partition_key);
    }

    pub fn remove_partition(&mut self, partition_key: &str) {
        self.partitions.remove(partition_key);
    }

    pub fn get_partition_and_update_last_access(
        &self,
        partition_key: &str,
        now: MyDateTime,
    ) -> Option<&DbPartition> {
        let result = self.partitions.get(partition_key)?;

        result.update_last_access(now);

        return Some(result);
    }

    pub fn get_partitions_amount(&self) -> usize {
        return self.partitions.len();
    }

    pub fn get_partition(&self, partition_key: &str) -> Option<&DbPartition> {
        return self.partitions.get(partition_key);
    }

    pub fn get_snapshot(&self) -> HashMap<String, Vec<Arc<DbRow>>> {
        let mut result = HashMap::new();

        for (partition_key, db_partition) in &self.partitions {
            let mut db_rows = Vec::new();

            for db_row in db_partition.rows.values() {
                db_rows.push(db_row.clone());
            }

            result.insert(partition_key.to_string(), db_rows);
        }

        result
    }

    pub fn get_all(&self) -> OperationResult {
        if self.partitions.len() == 0 {
            return OperationResult::Rows { rows: None };
        }

        let mut rows = Vec::new();

        for db_partition in self.partitions.values() {
            for db_row in db_partition.rows.values() {
                rows.push(db_row.clone());
            }
        }

        if rows.len() == 0 {
            return OperationResult::Rows { rows: None };
        }

        OperationResult::Rows { rows: Some(rows) }
    }

    pub fn get_by_row_key(&self, row_key: &str) -> OperationResult {
        if self.partitions.len() == 0 {
            return OperationResult::Rows { rows: None };
        }

        let mut rows = Vec::new();

        for db_partition in self.partitions.values() {
            let db_row = db_partition.rows.get(row_key);
            if let Some(db_row) = db_row {
                rows.push(db_row.clone());
            }
        }

        if rows.len() == 0 {
            return OperationResult::Rows { rows: None };
        }

        OperationResult::Rows { rows: Some(rows) }
    }

    pub fn get_row(
        &self,
        partition_key: &str,
        row_key: &str,
    ) -> Result<OperationResult, FailOperationResult> {
        let db_partition = self.partitions.get(partition_key);

        if db_partition.is_none() {
            return Err(FailOperationResult::RecordNotFound);
        }

        let db_partition = db_partition.unwrap();

        let db_row = db_partition.rows.get(row_key);

        if db_row.is_none() {
            return Err(FailOperationResult::RecordNotFound);
        }

        let db_row = db_row.unwrap();

        return Ok(OperationResult::Row {
            row: db_row.clone(),
        });
    }

    pub fn clear(&mut self) -> bool {
        if self.partitions.len() == 0 {
            return false;
        }

        self.partitions.clear();
        return true;
    }

    pub fn clear_partition(&mut self, partition_key: &str) -> bool {
        if !self.partitions.contains_key(partition_key) {
            return false;
        }

        self.partitions.remove(partition_key);
        return true;
    }

    pub fn set_table_attributes(
        &mut self,
        persist_table: bool,
        max_partitions_amount: Option<usize>,
    ) -> bool {
        if self.attributes.persist == persist_table
            && self.attributes.max_partitions_amount == max_partitions_amount
        {
            return false;
        }

        self.attributes.persist = persist_table;
        self.attributes.max_partitions_amount = max_partitions_amount;

        return true;
    }

    pub fn get_or_create_partition_and_update_last_access(
        &mut self,
        partition_key: &str,
        now: MyDateTime,
    ) -> &mut DbPartition {
        if self.partitions.contains_key(partition_key) {
            let result = self.partitions.get_mut(partition_key).unwrap();

            result.update_last_access(now);
            return result;
        }

        let result = DbPartition::new();

        self.partitions.insert(partition_key.to_string(), result);
        return self.partitions.get_mut(partition_key).unwrap();
    }

    pub async fn gc_partitions(&mut self, max_partitions_amount: usize) -> Vec<String> {
        let mut partitions_by_date_time: SortedHashMap<i64, String> = SortedHashMap::new();

        let mut gced = Vec::new();

        for (partition_key, db_partition) in &self.partitions {
            let mut last_access = db_partition.last_access;

            while partitions_by_date_time.contains_key(&last_access.miliseconds) {
                last_access.miliseconds += 1;
            }

            partitions_by_date_time.insert(last_access.miliseconds, partition_key.to_string());
        }

        while self.partitions.len() > max_partitions_amount {
            let (dt, partition_key) = partitions_by_date_time.first().unwrap();

            let removed_result = self.partitions.remove(&partition_key);

            if let Some(_) = removed_result {
                gced.push(partition_key);
            }

            partitions_by_date_time.remove(&dt);
        }

        gced
    }
}
