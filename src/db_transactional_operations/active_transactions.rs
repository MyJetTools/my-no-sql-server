use std::collections::HashMap;

use tokio::sync::Mutex;

use super::{TransactionalOperationStep, TransactionalOperations};

pub struct ActiveTransactions {
    items: Mutex<HashMap<String, TransactionalOperations>>,
}

impl ActiveTransactions {
    pub fn new() -> Self {
        Self {
            items: Mutex::new(HashMap::new()),
        }
    }

    pub async fn issue_new(&self) -> String {
        let mut write_access = self.items.lock().await;
        let id = generate_id(&write_access);
        write_access.insert(id.to_string(), TransactionalOperations::new());

        return id;
    }

    pub async fn add_events(&self, id: &str, mut events: Vec<TransactionalOperationStep>) -> bool {
        let mut write_access = self.items.lock().await;

        let result = write_access.get_mut(id);

        if result.is_none() {
            return false;
        }

        let transaction = result.unwrap();

        for event in events.drain(..) {
            transaction.add_event(event);
        }

        return true;
    }

    pub async fn remove(&self, id: &str) -> Option<TransactionalOperations> {
        let mut write_access = self.items.lock().await;
        write_access.remove(id)
    }
}

fn generate_id(items: &HashMap<String, TransactionalOperations>) -> String {
    loop {
        let id = uuid::Uuid::new_v4().to_string();

        if items.contains_key(&id) {
            return id;
        }
    }
}
