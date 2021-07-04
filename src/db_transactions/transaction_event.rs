use std::{collections::HashMap, sync::Arc};

use crate::db::{DbRow, DbTable};

use super::TransactionAttributes;

pub enum TransactionEvent {
    InitTable {
        table_name: String,
        attr: TransactionAttributes,
        partitions: HashMap<String, Vec<Arc<DbRow>>>,
    },

    CleanTable {
        table_name: String,
        attr: TransactionAttributes,
    },

    DeletePartitions {
        table_name: String,
        partitions: Vec<String>,
        attr: TransactionAttributes,
    },

    UpdateTableAttributes {
        table_name: String,
        attr: TransactionAttributes,
        persist: bool,
        max_partitions_amount: Option<usize>,
    },

    UpdateRow {
        table_name: String,
        attr: TransactionAttributes,
        partition_key: String,
        row: Arc<DbRow>,
    },

    UpdateRows {
        table_name: String,
        attr: TransactionAttributes,
        rows_by_partition: HashMap<String, Vec<Arc<DbRow>>>,
    },

    DeleteRows {
        table_name: String,
        attr: TransactionAttributes,
        rows: HashMap<String, Vec<String>>,
    },
}

impl TransactionEvent {
    pub fn init_table(
        table_name: String,
        attr: TransactionAttributes,
        partitions: HashMap<String, Vec<Arc<DbRow>>>,
    ) -> Self {
        Self::InitTable {
            attr,
            table_name,
            partitions,
        }
    }

    pub fn update_row(
        table: &DbTable,
        attr: TransactionAttributes,
        partition_key: &str,
        row: Arc<DbRow>,
    ) -> Self {
        let mut rows_by_partition = HashMap::new();

        rows_by_partition.insert(partition_key.to_string(), vec![row]);

        Self::UpdateRows {
            table_name: table.name.to_string(),
            attr: attr,
            rows_by_partition,
        }
    }

    pub fn update_rows(
        table: &DbTable,
        attr: TransactionAttributes,
        rows_by_partition: HashMap<String, Vec<Arc<DbRow>>>,
    ) -> Self {
        Self::UpdateRows {
            table_name: table.name.to_string(),
            attr: attr,
            rows_by_partition,
        }
    }

    pub fn get_table_name(&self) -> &str {
        match self {
            TransactionEvent::InitTable {
                table_name,
                attr: _,
                partitions: _,
            } => table_name,
            TransactionEvent::CleanTable {
                table_name,
                attr: _,
            } => table_name,
            TransactionEvent::UpdateTableAttributes {
                table_name,
                attr: _,
                persist: _,
                max_partitions_amount: _,
            } => table_name,
            TransactionEvent::UpdateRow {
                table_name,
                attr: _,
                partition_key: _,
                row: _,
            } => table_name,
            TransactionEvent::UpdateRows {
                table_name,
                attr: _,
                rows_by_partition: _,
            } => table_name,

            TransactionEvent::DeletePartitions {
                table_name,
                attr: _,
                partitions: _,
            } => &table_name,
            TransactionEvent::DeleteRows {
                table_name,
                attr: _,
                rows: _,
            } => table_name,
        }
    }
}
