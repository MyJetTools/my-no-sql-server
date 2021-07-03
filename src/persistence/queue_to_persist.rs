use std::{collections::HashMap, sync::Arc};

use tokio::sync::Mutex;

use crate::db_transactions::TransactionEvent;

use super::queue_by_table::QueueByTable;

pub struct QueueToPersist {
    queue: Mutex<HashMap<String, QueueByTable>>,
}

impl QueueToPersist {
    pub fn new() -> Self {
        Self {
            queue: Mutex::new(HashMap::new()),
        }
    }

    pub async fn enqueue(&self, event: Arc<TransactionEvent>) {
        let table_name = event.get_table_name();

        let mut queue = self.queue.lock().await;

        let queue_by_table = queue.get_mut(table_name);

        if queue_by_table.is_none() {
            let queue_by_table = QueueByTable::new(table_name.to_string());
            queue.insert(table_name.to_string(), queue_by_table);
        }

        let queue_by_table = queue.get_mut(table_name).unwrap();

        queue_by_table.enqueue(event);
    }

    pub async fn dequeue(&self) -> Option<(String, Vec<Arc<TransactionEvent>>)> {
        let mut queue = self.queue.lock().await;

        for queue_by_table in queue.values_mut() {
            let next_elements = queue_by_table.dequeue();

            if next_elements.is_none() {
                continue;
            }

            let next_elementes = next_elements.unwrap();

            return Some((queue_by_table.table_name.to_string(), next_elementes));
        }

        None
    }
}
