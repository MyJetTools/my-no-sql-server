use std::collections::BTreeMap;

use super::steps::TransactionalOperationStep;

pub struct TransactionalOperations {
    pub operations: BTreeMap<String, Vec<TransactionalOperationStep>>,
}

impl TransactionalOperations {
    pub fn new() -> Self {
        Self {
            operations: BTreeMap::new(),
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
