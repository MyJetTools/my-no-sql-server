use std::sync::Arc;

use my_no_sql_sdk::core::rust_extensions::{
    events_loop::{EventsLoop, EventsLoopPublisher, EventsLoopTick},
    ApplicationStates,
};
use tokio::sync::Mutex;

use crate::db_sync::SyncEvent;

pub struct EventsSync {
    sync: Mutex<EventsLoop<SyncEvent>>,
    pub publisher: EventsLoopPublisher<SyncEvent>,
}

impl EventsSync {
    pub fn new() -> Self {
        let mut sync = EventsLoop::new("Sync".to_string(), my_logger::LOGGER.clone());
        let publisher = sync.get_publisher();
        Self {
            sync: Mutex::new(sync),
            publisher,
        }
    }

    pub async fn register_event_loop(
        &self,
        events_loop: Arc<dyn EventsLoopTick<SyncEvent> + Send + Sync + 'static>,
    ) {
        let mut write_access = self.sync.lock().await;
        write_access.register_event_loop(events_loop);
    }

    pub async fn start(&self, app_states: Arc<dyn ApplicationStates + Send + Sync + 'static>) {
        let mut write_access = self.sync.lock().await;
        write_access.start(app_states);
    }
}
