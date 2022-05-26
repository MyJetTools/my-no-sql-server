use std::{
    sync::{atomic::AtomicUsize, Arc},
    time::Duration,
};

use rust_extensions::{date_time::DateTimeAsMicroseconds, ApplicationStates, MyTimerLogger};

use crate::{
    data_readers::DataReadersList,
    db::DbInstance,
    db_operations::multipart::MultipartList,
    db_transactions::ActiveTransactions,
    persist_io::PersistIoOperations,
    persist_operations::{blob_content_cache::BlobContentCache, data_initializer::InitState},
    settings_reader::SettingsModel,
};

use super::{
    global_states::GlobalStates,
    logs::{Logs, SystemProcess},
    EventsDispatcher, PrometheusMetrics,
};

pub const APP_VERSION: &'static str = env!("CARGO_PKG_VERSION");

pub const DEFAULT_PERSIST_PERIOD: crate::db_sync::DataSynchronizationPeriod =
    crate::db_sync::DataSynchronizationPeriod::Sec5;

pub struct AppContext {
    pub created: DateTimeAsMicroseconds,
    pub db: DbInstance,
    pub logs: Arc<Logs>,

    pub metrics: PrometheusMetrics,

    pub active_transactions: ActiveTransactions,
    pub process_id: String,

    pub states: GlobalStates,

    pub events_dispatcher: Box<dyn EventsDispatcher + Send + Sync + 'static>,
    pub blob_content_cache: BlobContentCache,
    pub data_readers: DataReadersList,

    pub multipart_list: MultipartList,
    pub persist_io: Arc<dyn PersistIoOperations + Send + Sync + 'static>,
    pub init_state: InitState,
    pub settings: Arc<SettingsModel>,
    persist_amount: AtomicUsize,
}

impl AppContext {
    pub fn new(
        logs: Arc<Logs>,
        settings: Arc<SettingsModel>,
        events_dispatcher: Box<dyn EventsDispatcher + Send + Sync + 'static>,
        persist_io: Arc<dyn PersistIoOperations + Send + Sync + 'static>,
    ) -> Self {
        AppContext {
            created: DateTimeAsMicroseconds::now(),
            init_state: InitState::new(),
            db: DbInstance::new(),
            logs,
            metrics: PrometheusMetrics::new(),
            active_transactions: ActiveTransactions::new(),
            process_id: uuid::Uuid::new_v4().to_string(),
            states: GlobalStates::new(),

            events_dispatcher,
            blob_content_cache: BlobContentCache::new(),
            data_readers: DataReadersList::new(Duration::from_secs(30)),
            multipart_list: MultipartList::new(),
            persist_io,
            settings,
            persist_amount: AtomicUsize::new(0),
        }
    }

    pub fn update_persist_amount(&self, value: usize) {
        self.persist_amount
            .store(value, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn get_persist_amount(&self) -> usize {
        self.persist_amount
            .load(std::sync::atomic::Ordering::Relaxed)
    }
}

impl ApplicationStates for AppContext {
    fn is_initialized(&self) -> bool {
        self.states.is_initialized()
    }

    fn is_shutting_down(&self) -> bool {
        self.states.is_shutting_down()
    }
}

impl MyTimerLogger for AppContext {
    fn write_info(&self, timer_id: String, message: String) {
        self.logs
            .add_info(None, SystemProcess::Timer, timer_id, message);
    }

    fn write_error(&self, timer_id: String, message: String) {
        self.logs
            .add_fatal_error(None, SystemProcess::Timer, timer_id, message);
    }
}
