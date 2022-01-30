use std::collections::HashMap;

use tokio::sync::{mpsc::UnboundedSender, Mutex};

use crate::db_sync::SyncEvent;

pub struct NextEventsToHandle {
    pub table_name: String,
    pub events: Vec<SyncEvent>,
}

pub struct EventsDispatcher {
    pub events: Mutex<HashMap<String, Vec<SyncEvent>>>,
    sender: Option<UnboundedSender<()>>,
}

impl EventsDispatcher {
    pub fn new(sender: Option<UnboundedSender<()>>) -> Self {
        Self {
            events: Mutex::new(HashMap::new()),
            sender,
        }
    }

    pub async fn dispatch(&self, event: SyncEvent) {
        {
            let table_name = event.get_table_name();
            let mut write_access = self.events.lock().await;

            if !write_access.contains_key(table_name) {
                write_access.insert(table_name.to_string(), Vec::new());
            }

            write_access.get_mut(table_name).unwrap().push(event);
        }

        if let Some(sender) = &self.sender {
            let result = sender.send(());

            if let Err(err) = result {
                println!("Error on dispatching event. Err: {:?}", err);
            }
        }
    }

    pub async fn get_next_events(&self) -> Option<NextEventsToHandle> {
        let mut write_access = self.events.lock().await;

        for itm in write_access.drain() {
            return Some(NextEventsToHandle {
                table_name: itm.0,
                events: itm.1,
            });
        }

        return None;
    }
}
