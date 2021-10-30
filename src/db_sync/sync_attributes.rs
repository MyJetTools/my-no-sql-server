use std::collections::HashMap;

#[derive(Clone, Copy)]
pub enum EventSource {
    ClientRequest,
    GarbageCollector,
    // Synchronization, //TODO - when we doing nodes support - we restore these
    // Init
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

#[derive(Clone)]
pub struct SyncAttributes {
    pub locations: Vec<String>,
    pub sync_period: DataSynchronizationPeriod,
    pub event_source: EventSource,
    pub headers: Option<HashMap<String, String>>,
}
