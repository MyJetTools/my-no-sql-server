use std::{collections::HashMap, sync::Arc};

use crate::db_transactions::TransactionEvent;

pub struct BlobOperationsToBeMade {
    pub sync_attributes: bool,
    pub sync_table: bool,
    pub sync_partitions: HashMap<String, bool>,
    pub delete_table: bool, //TODO - Make different saving optimization
}

impl BlobOperationsToBeMade {
    pub fn new(transactions: &[Arc<TransactionEvent>]) -> Self {
        let mut result = Self {
            sync_attributes: false,
            sync_partitions: HashMap::new(),
            sync_table: false,
            delete_table: false,
        };

        for event in transactions {
            match event.as_ref() {
                TransactionEvent::InitTable {
                    table: _,
                    attr: _,
                    partitions: _,
                } => result.sync_table = true,
                TransactionEvent::CleanTable { table: _, attr: _ } => result.sync_table = true,
                TransactionEvent::DeletePartitions {
                    table: _,
                    partitions,
                    attr: _,
                } => {
                    for partition_key in partitions {
                        if result.sync_partitions.contains_key(partition_key) {
                            result
                                .sync_partitions
                                .insert(partition_key.to_string(), true);
                        }
                    }
                }
                TransactionEvent::UpdateTableAttributes {
                    table: _,
                    attr: _,
                    table_is_just_created: _,
                    persist: _,
                    max_partitions_amount: _,
                } => {
                    result.sync_attributes = true;
                }
                TransactionEvent::UpdateRow {
                    table: _,
                    attr: _,
                    partition_key,
                    row: _,
                } => {
                    if result.sync_partitions.contains_key(partition_key) {
                        result
                            .sync_partitions
                            .insert(partition_key.to_string(), true);
                    }
                }
                TransactionEvent::UpdateRows {
                    table: _,
                    attr: _,
                    rows_by_partition,
                } => {
                    for (partition_key, _) in rows_by_partition {
                        if result.sync_partitions.contains_key(partition_key) {
                            result
                                .sync_partitions
                                .insert(partition_key.to_string(), true);
                        }
                    }
                }
                TransactionEvent::DeleteRows {
                    table: _,
                    attr: _,
                    rows,
                } => {
                    for (partition_key, _) in rows {
                        if result.sync_partitions.contains_key(partition_key) {
                            result
                                .sync_partitions
                                .insert(partition_key.to_string(), true);
                        }
                    }
                }
                TransactionEvent::DeleteTable { table: _, attr: _ } => result.delete_table = true,
            }
        }

        return result;
    }
}
