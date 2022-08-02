use std::collections::HashMap;

use rust_extensions::date_time::DateTimeAsMicroseconds;

#[derive(Debug, Clone)]
pub enum PartitionPersistData {
    WholePartition(DateTimeAsMicroseconds),
    Rows(HashMap<String, DateTimeAsMicroseconds>),
}

impl PartitionPersistData {
    pub fn is_rows(&self) -> bool {
        match self {
            PartitionPersistData::WholePartition(_) => false,
            PartitionPersistData::Rows(_) => true,
        }
    }

    pub fn unwrap_as_partition_persist_moment(&self) -> DateTimeAsMicroseconds {
        match self {
            PartitionPersistData::WholePartition(value) => *value,
            PartitionPersistData::Rows(_) => {
                panic!("Result is not as partition");
            }
        }
    }

    pub fn get_min_persist_time(&self) -> DateTimeAsMicroseconds {
        match self {
            PartitionPersistData::WholePartition(date_time) => *date_time,
            PartitionPersistData::Rows(rows) => {
                let mut result: Option<DateTimeAsMicroseconds> = None;

                for row_date_time in rows.values() {
                    if result.is_none() {
                        result = Some(*row_date_time);
                    } else if row_date_time.unix_microseconds < result.unwrap().unix_microseconds {
                        result = Some(row_date_time.clone());
                    }
                }

                result.unwrap()
            }
        }
    }
}
