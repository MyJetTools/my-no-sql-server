use std::{collections::HashMap, sync::Arc};

use my_no_sql_core::db::DbRow;
use rust_extensions::date_time::DateTimeAsMicroseconds;

use super::{PartitionPersistData, PersistResult};

pub struct DataToPersist {
    pub persisit_whole_table: Option<DateTimeAsMicroseconds>,
    pub partitions: HashMap<String, PartitionPersistData>,
    pub persist_attrs: bool,
}

impl DataToPersist {
    pub fn get_persist_amount(&self) -> usize {
        let mut result = if self.persist_attrs { 1 } else { 0 };
        result += self.partitions.len();

        if self.persisit_whole_table.is_some() {
            result += 1;
        };

        result
    }

    pub fn new() -> Self {
        Self {
            persisit_whole_table: None,
            partitions: HashMap::new(),
            persist_attrs: false,
        }
    }

    pub fn mark_table_to_persist(&mut self, moment: DateTimeAsMicroseconds) {
        if self.persisit_whole_table.is_none() {
            self.persisit_whole_table = Some(moment);
            return;
        }

        let persist_whole_table = self.persisit_whole_table.unwrap();

        if persist_whole_table.unix_microseconds > moment.unix_microseconds {
            self.persisit_whole_table = Some(moment)
        }
    }

    pub fn mark_row_to_persit(
        &mut self,
        partition_key: &str,
        row_key: &str,
        new_persist_moment: DateTimeAsMicroseconds,
    ) {
        if !self.partitions.contains_key(partition_key) {
            self.partitions.insert(
                partition_key.to_string(),
                PartitionPersistData::Rows(HashMap::new()),
            );
        }

        match self.partitions.get_mut(partition_key).unwrap() {
            PartitionPersistData::WholePartition(date_time) => {
                if new_persist_moment.unix_microseconds < date_time.unix_microseconds {
                    date_time.unix_microseconds = new_persist_moment.unix_microseconds;
                }
            }
            PartitionPersistData::Rows(rows) => {
                if !rows.contains_key(row_key) {
                    rows.insert(row_key.to_string(), new_persist_moment);
                }
            }
        }
    }

    pub fn mark_rows_to_persit(
        &mut self,
        partition_key: &str,
        rows: &[Arc<DbRow>],
        new_persist_moment: DateTimeAsMicroseconds,
    ) {
        for db_row in rows {
            self.mark_row_to_persit(partition_key, &db_row.row_key.as_str(), new_persist_moment);
        }
    }

    pub fn mark_row_keys_to_persit(
        &mut self,
        partition_key: &str,
        row_keys: &[String],
        new_persist_moment: DateTimeAsMicroseconds,
    ) {
        for row_key in row_keys {
            self.mark_row_to_persit(partition_key, row_key.as_str(), new_persist_moment);
        }
    }

    pub fn mark_partition_to_persist(
        &mut self,
        partition_key: &str,
        persist_moment: DateTimeAsMicroseconds,
    ) {
        if !self.partitions.contains_key(partition_key) {
            self.partitions.insert(
                partition_key.to_string(),
                PartitionPersistData::WholePartition(persist_moment),
            );
            return;
        }

        let upgrade_to_partition = self.partitions.get(partition_key).unwrap().is_rows();

        if upgrade_to_partition {
            self.partitions.insert(
                partition_key.to_string(),
                PartitionPersistData::WholePartition(persist_moment),
            );
            return;
        }

        let partition_persist_moment = self
            .partitions
            .get(partition_key)
            .unwrap()
            .unwrap_as_partition_persist_moment();

        if partition_persist_moment.unix_microseconds > persist_moment.unix_microseconds {
            self.partitions.insert(
                partition_key.to_string(),
                PartitionPersistData::WholePartition(persist_moment),
            );
        }
    }

    pub fn mark_persist_attrs(&mut self) {
        self.persist_attrs = true;
    }

    fn get_next_partition_result(&mut self) -> Option<PersistResult> {
        for (partition_key, persist_data) in &self.partitions {
            match persist_data {
                PartitionPersistData::WholePartition(persist_moment) => {
                    return Some(PersistResult::PersistPartition {
                        partition_key: partition_key.to_string(),
                        persist_moment: *persist_moment,
                    });
                }
                PartitionPersistData::Rows(rows) => {
                    let result = PersistResult::PersistRows {
                        partition_key: partition_key.to_string(),
                        row_keys: rows.clone(),
                    };

                    return Some(result);
                }
            }
        }

        None
    }

    pub fn get_what_to_persist(&mut self) -> Option<PersistResult> {
        if let Some(persisit_whole_table) = self.persisit_whole_table {
            self.persisit_whole_table = None;
            self.partitions.clear();
            return Some(PersistResult::PersistTable(persisit_whole_table));
        }

        if self.persist_attrs {
            self.persist_attrs = false;
            return Some(PersistResult::PersistAttrs);
        }

        let result = self.get_next_partition_result()?;

        if let Some(partition_key) = result.get_partition_key() {
            self.partitions.remove(partition_key);
        }

        Some(result)
    }

    pub fn get_next_persist_time(&self) -> Option<DateTimeAsMicroseconds> {
        let mut result: Option<DateTimeAsMicroseconds> = None;

        if let Some(whole_table_date_time) = self.persisit_whole_table {
            match &mut result {
                Some(result_date_time) => {
                    if result_date_time.unix_microseconds > whole_table_date_time.unix_microseconds
                    {
                        result_date_time.unix_microseconds =
                            whole_table_date_time.unix_microseconds;
                    }
                }
                None => result = Some(whole_table_date_time),
            }
        }

        for row_date_time in self.partitions.values() {
            match &mut result {
                Some(result_date_time) => {
                    if result_date_time.unix_microseconds
                        > row_date_time.get_min_persist_time().unix_microseconds
                    {
                        result_date_time.unix_microseconds =
                            row_date_time.get_min_persist_time().unix_microseconds;
                    }
                }
                None => result = Some(row_date_time.get_min_persist_time()),
            }
        }

        result
    }
}
