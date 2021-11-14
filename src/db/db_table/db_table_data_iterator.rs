use std::{collections::VecDeque, sync::Arc};

use crate::db::{DbPartition, DbRow};

use super::db_table_data::TPartitions;

pub struct DbTableDataIterator<'s> {
    partitions: VecDeque<&'s DbPartition>,
    current_partition: Option<VecDeque<&'s Arc<DbRow>>>,
}

impl<'s> DbTableDataIterator<'s> {
    pub fn new(src_partitions: &'s TPartitions) -> Self {
        let mut partitions = VecDeque::new();

        for partition in src_partitions.values() {
            partitions.push_front(partition);
        }

        Self {
            partitions,
            current_partition: None,
        }
    }
}

impl<'s> Iterator for DbTableDataIterator<'s> {
    type Item = &'s Arc<DbRow>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(current_partition) = &mut self.current_partition {
                let next_db_row_result = current_partition.remove(0);

                if let Some(db_row) = next_db_row_result {
                    return Some(db_row);
                }
            }

            let next_partition = self.partitions.remove(0);

            if next_partition.is_none() {
                return None;
            }

            let next_partition = next_partition.unwrap();

            let db_rows = next_partition.rows.values().collect();

            self.current_partition = Some(db_rows);
        }
    }
}
