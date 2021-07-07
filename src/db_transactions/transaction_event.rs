use std::{collections::HashMap, sync::Arc};

use crate::db::{DbRow, DbTable};

use super::TransactionAttributes;

pub enum TransactionEvent {
    UpdateTableAttributes {
        table: Arc<DbTable>,
        attr: TransactionAttributes,
        table_is_just_created: bool,
        persist: bool,
        max_partitions_amount: Option<usize>,
    },
    InitTable {
        table: Arc<DbTable>,
        attr: TransactionAttributes,
        partitions: HashMap<String, Vec<Arc<DbRow>>>,
    },

    DeleteTable {
        table: Arc<DbTable>,
        attr: TransactionAttributes,
    },

    CleanTable {
        table: Arc<DbTable>,
        attr: TransactionAttributes,
    },

    DeletePartitions {
        table: Arc<DbTable>,
        partitions: Vec<String>,
        attr: TransactionAttributes,
    },

    UpdateRow {
        table: Arc<DbTable>,
        attr: TransactionAttributes,
        partition_key: String,
        row: Arc<DbRow>,
    },

    UpdateRows {
        table: Arc<DbTable>,
        attr: TransactionAttributes,
        rows_by_partition: HashMap<String, Vec<Arc<DbRow>>>,
    },

    DeleteRows {
        table: Arc<DbTable>,
        attr: TransactionAttributes,
        rows: HashMap<String, Vec<String>>,
    },
}

impl TransactionEvent {
    pub fn init_table(
        table: Arc<DbTable>,
        attr: TransactionAttributes,
        partitions: HashMap<String, Vec<Arc<DbRow>>>,
    ) -> Self {
        Self::InitTable {
            attr,
            table,
            partitions,
        }
    }

    pub fn update_row(
        table: Arc<DbTable>,
        attr: TransactionAttributes,
        partition_key: &str,
        row: Arc<DbRow>,
    ) -> Self {
        let mut rows_by_partition = HashMap::new();

        rows_by_partition.insert(partition_key.to_string(), vec![row]);

        Self::UpdateRows {
            table,
            attr: attr,
            rows_by_partition,
        }
    }

    pub fn update_rows(
        table: Arc<DbTable>,
        attr: TransactionAttributes,
        rows_by_partition: HashMap<String, Vec<Arc<DbRow>>>,
    ) -> Self {
        Self::UpdateRows {
            table,
            attr: attr,
            rows_by_partition,
        }
    }
}
