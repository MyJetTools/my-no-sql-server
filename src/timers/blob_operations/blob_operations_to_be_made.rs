use std::{collections::HashMap, sync::Arc};

use crate::db_transactions::TransactionEvent;

pub struct BlobOperationsToBeMade {
    pub sync_attributes: bool,
    pub sync_table: bool,
    pub sync_partitions: HashMap<String, bool>,
}

impl BlobOperationsToBeMade {
    pub fn new(transactions: &[Arc<TransactionEvent>]) -> Self {
        let mut result = Self {
            sync_attributes: false,
            sync_partitions: HashMap::new(),
            sync_table: false,
        };

        for event in transactions {
            match event.as_ref() {
                TransactionEvent::InitTable {
                    table_name: _,
                    attr: _,
                    partitions: _,
                } => result.sync_table = true,
                TransactionEvent::CleanTable {
                    table_name: _,
                    attr: _,
                } => result.sync_table = true,
                TransactionEvent::DeletePartitions {
                    table_name: _,
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
                    table_name: _,
                    attr: _,
                    persist: _,
                    max_partitions_amount: _,
                } => {
                    result.sync_attributes = true;
                }
                TransactionEvent::UpdateRow {
                    table_name: _,
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
                    table_name: _,
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
                    table_name: _,
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
            }
        }

        return result;
    }
}
