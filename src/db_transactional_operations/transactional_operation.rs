use std::{collections::HashMap, sync::Arc};

use crate::db::DbRow;

pub enum TransactionalOperationStep {
    CleanTable {
        table_name: String,
    },
    DeletePartitions {
        table_name: String,
        partition_keys: Vec<String>,
    },

    DeleteRows {
        table_name: String,
        partition_key: String,
        row_keys: Vec<String>,
    },

    UpdateRows {
        table_name: String,
        rows_by_partition: HashMap<String, Vec<Arc<DbRow>>>,
    },
}

impl TransactionalOperationStep {
    pub fn get_table_name(&self) -> &str {
        match self {
            TransactionalOperationStep::CleanTable { table_name } => table_name,
            TransactionalOperationStep::DeletePartitions {
                table_name,
                partition_keys: _,
            } => table_name,
            TransactionalOperationStep::DeleteRows {
                table_name,
                partition_key: _,
                row_keys: _,
            } => table_name,
            TransactionalOperationStep::UpdateRows {
                table_name,
                rows_by_partition: _,
            } => table_name,
        }
    }
}

pub struct TransactionalOperations {
    pub operations: HashMap<String, Vec<TransactionalOperationStep>>,
}

impl TransactionalOperations {
    pub fn new() -> Self {
        Self {
            operations: HashMap::new(),
        }
    }

    pub fn add_event(&mut self, event: TransactionalOperationStep) {
        let table_name = event.get_table_name();

        if !self.operations.contains_key(table_name) {
            self.operations.insert(table_name.to_string(), Vec::new());
        }

        self.operations.get_mut(table_name).unwrap().push(event);
    }
}
