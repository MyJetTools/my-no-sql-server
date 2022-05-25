use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use rust_extensions::date_time::DateTimeAsMicroseconds;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::{db::DbTable, db_sync::SyncEvent};

pub trait EventsDispatcher {
    fn dispatch(&self, db_table: Option<&DbTable>, event: SyncEvent);
    fn get_events_queue_size(&self) -> usize;
}

pub struct EventsDispatcherProduction {
    size: Arc<AtomicUsize>,

    events_receiver: Option<UnboundedReceiver<SyncEvent>>,
    events_sender: UnboundedSender<SyncEvent>,
}

impl EventsDispatcherProduction {
    pub fn new() -> Self {
        let (events_sender, events_receiver) = tokio::sync::mpsc::unbounded_channel();
        Self {
            size: Arc::new(AtomicUsize::new(0)),
            events_sender,
            events_receiver: Some(events_receiver),
        }
    }

    pub fn get_events_reader(&mut self) -> SyncEventsReader {
        let mut events_receiver = None;
        std::mem::swap(&mut events_receiver, &mut self.events_receiver);

        if events_receiver.is_none() {
            println!("You can get events reader only ones");
        }

        SyncEventsReader {
            size: self.size.clone(),
            events_receiver: events_receiver.unwrap(),
        }
    }
}

impl EventsDispatcher for EventsDispatcherProduction {
    fn dispatch(&self, db_table: Option<&DbTable>, event: SyncEvent) {
        if let Some(db_table) = db_table {
            db_table.set_last_update_time(DateTimeAsMicroseconds::now());
        }

        let result = self.events_sender.send(event);
        self.size.fetch_add(1, Ordering::SeqCst);
        if let Err(_) = result {
            println!("Error on dispatching event.");
        }
    }

    fn get_events_queue_size(&self) -> usize {
        self.size.load(Ordering::Relaxed)
    }
}

pub struct SyncEventsReader {
    size: Arc<AtomicUsize>,
    events_receiver: UnboundedReceiver<SyncEvent>,
}

impl SyncEventsReader {
    pub async fn get_next_event(&mut self) -> Option<SyncEvent> {
        let result = self.events_receiver.recv().await;
        if result.is_some() {
            self.size.fetch_sub(1, Ordering::SeqCst);
        }

        result
    }
}
