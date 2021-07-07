use std::sync::Arc;

use tokio::sync::Mutex;

use crate::db_transactions::TransactionEvent;

pub struct QueueToPersist {
    queue: Mutex<Vec<Arc<TransactionEvent>>>,
}

impl QueueToPersist {
    pub fn new() -> Self {
        Self {
            queue: Mutex::new(Vec::new()),
        }
    }

    pub async fn enqueue(&self, event: Arc<TransactionEvent>) {
        let mut queue = self.queue.lock().await;

        queue.push(event);
    }

    pub async fn dequeue(&self) -> Option<Arc<TransactionEvent>> {
        let mut queue = self.queue.lock().await;

        if queue.len() == 0 {
            return None;
        }

        return Some(queue.remove(0));
    }
}
