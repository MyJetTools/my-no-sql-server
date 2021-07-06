use std::{collections::HashMap, sync::Arc};

use crate::{
    date_time::MyDateTime,
    db::{DbRow, DbTable},
};

use super::TransactionAttributes;

pub struct TransactionTableInfo {
    pub name: String,
    pub created: MyDateTime,
}

impl From<&DbTable> for TransactionTableInfo {
    fn from(src: &DbTable) -> Self {
        Self {
            name: src.name.to_string(),
            created: src.created,
        }
    }
}

pub enum TransactionEvent {
    InitTable {
        table: TransactionTableInfo,
        attr: TransactionAttributes,
        partitions: HashMap<String, Vec<Arc<DbRow>>>,
    },

    DeleteTable {
        table: TransactionTableInfo,
        attr: TransactionAttributes,
    },

    CleanTable {
        table: TransactionTableInfo,
        attr: TransactionAttributes,
    },

    DeletePartitions {
        table: TransactionTableInfo,
        partitions: Vec<String>,
        attr: TransactionAttributes,
    },

    UpdateTableAttributes {
        table: TransactionTableInfo,
        attr: TransactionAttributes,
        persist: bool,
        max_partitions_amount: Option<usize>,
    },

    UpdateRow {
        table: TransactionTableInfo,
        attr: TransactionAttributes,
        partition_key: String,
        row: Arc<DbRow>,
    },

    UpdateRows {
        table: TransactionTableInfo,
        attr: TransactionAttributes,
        rows_by_partition: HashMap<String, Vec<Arc<DbRow>>>,
    },

    DeleteRows {
        table: TransactionTableInfo,
        attr: TransactionAttributes,
        rows: HashMap<String, Vec<String>>,
    },
}

impl TransactionEvent {
    pub fn init_table(
        table: &DbTable,
        attr: TransactionAttributes,
        partitions: HashMap<String, Vec<Arc<DbRow>>>,
    ) -> Self {
        Self::InitTable {
            attr,
            table: table.into(),
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
            table: table.into(),
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
            table: table.into(),
            attr: attr,
            rows_by_partition,
        }
    }

    pub fn get_table_name(&self) -> &str {
        match self {
            TransactionEvent::InitTable {
                table,
                attr: _,
                partitions: _,
            } => table.name.as_str(),
            TransactionEvent::CleanTable { table, attr: _ } => table.name.as_str(),
            TransactionEvent::UpdateTableAttributes {
                table,
                attr: _,
                persist: _,
                max_partitions_amount: _,
            } => table.name.as_str(),
            TransactionEvent::UpdateRow {
                table,
                attr: _,
                partition_key: _,
                row: _,
            } => table.name.as_str(),
            TransactionEvent::UpdateRows {
                table,
                attr: _,
                rows_by_partition: _,
            } => table.name.as_str(),

            TransactionEvent::DeletePartitions {
                table,
                attr: _,
                partitions: _,
            } => table.name.as_str(),
            TransactionEvent::DeleteRows {
                table,
                attr: _,
                rows: _,
            } => table.name.as_str(),
            TransactionEvent::DeleteTable { table, attr: _ } => table.name.as_str(),
        }
    }
}
