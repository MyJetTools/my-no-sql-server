use std::collections::HashMap;

#[derive(Clone, Copy)]
pub enum EventSource {
    ClientRequest,
    // Synchronization, //TODO - when we doing nodes support - we restore these
    // Init,
}

#[derive(Clone, Copy)]
pub enum DataSynchronizationPeriod {
    Immediately,
    Sec1,
    Sec5,
    Sec15,
    Sec30,
    Min1,
    Asap,
}

pub struct TransactionAttributes {
    pub locations: Vec<String>,
    pub sync_period: DataSynchronizationPeriod,
    pub event_source: EventSource,
    pub headers: Option<HashMap<String, String>>,
}

impl Clone for TransactionAttributes {
    fn clone(&self) -> Self {
        Self {
            locations: self.locations.clone(),
            sync_period: self.sync_period,
            event_source: self.event_source,
            headers: self.headers.clone(),
        }
    }
}
