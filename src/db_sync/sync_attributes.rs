use std::collections::HashMap;

use my_http_macros::MyHttpStringEnum;

#[derive(Clone, Copy)]
pub enum EventSource {
    ClientRequest,
    GarbageCollector,
    // Synchronization, //TODO - when we doing nodes support - we restore these
    // Init
}

#[derive(Clone, Copy, MyHttpStringEnum)]
pub enum DataSynchronizationPeriod {
    #[http_enum_case(id="0"; description="Immediately sync")]
    Immediately,
    #[http_enum_case(id="1"; description="Sync during 1 sec")]
    Sec1,
    #[http_enum_case(id="2"; description="Sync during 5 sec")]
    Sec5,
    #[http_enum_case(id="3"; description="Sync during 15 sec")]
    Sec15,
    #[http_enum_case(id="4"; description="Sync during 30 sec")]
    Sec30,
    #[http_enum_case(id="5"; description="Sync during 1 minute")]
    Min1,
    #[http_enum_case(id="6"; description="Sync as soon as CPU schedules task")]
    Asap,
}

impl DataSynchronizationPeriod {
    pub fn as_str(&self) -> &str {
        match self {
            DataSynchronizationPeriod::Immediately => "Immediately",
            DataSynchronizationPeriod::Sec1 => "1 second",
            DataSynchronizationPeriod::Sec5 => "5 seconds",
            DataSynchronizationPeriod::Sec15 => "15 seconds",
            DataSynchronizationPeriod::Sec30 => "30 seconds",
            DataSynchronizationPeriod::Min1 => "1 minute",
            DataSynchronizationPeriod::Asap => "As soon as possible",
        }
    }
}

#[derive(Clone)]
pub struct SyncAttributes {
    pub locations: Vec<String>,
    pub sync_period: DataSynchronizationPeriod,
    pub event_source: EventSource,
    pub headers: Option<HashMap<String, String>>,
}
