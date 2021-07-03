use std::{mem, sync::Arc};

use crate::db_transactions::TransactionEvent;

pub struct QueueByTable {
    pub table_name: String,
    queue: Vec<Arc<TransactionEvent>>,
}

impl QueueByTable {
    pub fn new(table_name: String) -> Self {
        Self {
            queue: Vec::new(),
            table_name,
        }
    }

    pub fn enqueue(&mut self, event: Arc<TransactionEvent>) {
        self.queue.push(event);
    }

    pub fn dequeue(&mut self) -> Option<Vec<Arc<TransactionEvent>>> {
        if self.queue.len() == 0 {
            return None;
        }

        let mut result = Vec::new();

        mem::swap(&mut result, &mut self.queue);

        return Some(result);
    }
}
