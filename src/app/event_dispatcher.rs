use tokio::sync::mpsc::UnboundedSender;

use crate::db_sync::SyncEvent;

pub trait EventsDispatcher {
    fn dispatch(&self, event: SyncEvent);
}

pub struct EventsDispatcherProduction {
    sender: UnboundedSender<SyncEvent>,
}

impl EventsDispatcherProduction {
    pub fn new(sender: UnboundedSender<SyncEvent>) -> Self {
        Self { sender }
    }
}

impl EventsDispatcher for EventsDispatcherProduction {
    fn dispatch(&self, event: SyncEvent) {
        let result = self.sender.send(event);

        if let Err(_) = result {
            println!("Error on dispatching event.");
        }
    }
}
