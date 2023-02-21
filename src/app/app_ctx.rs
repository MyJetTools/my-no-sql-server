use std::{
    collections::HashMap,
    sync::{atomic::AtomicUsize, Arc},
    time::Duration,
};

use my_no_sql_server_core::{logs::*, DbInstance};
use rust_extensions::{
    date_time::DateTimeAsMicroseconds, events_loop::EventsLoop, AppStates, Logger,
};

use crate::{
    data_readers::DataReadersList,
    db_operations::multipart::MultipartList,
    db_sync::SyncEvent,
    db_transactions::ActiveTransactions,
    persist::PersistMarkersByTable,
    persist_io::PersistIoOperations,
    persist_operations::{
        blob_content_cache::BlobContentCache, data_initializer::load_tasks::InitState,
    },
    settings_reader::SettingsModel,
};

use super::PrometheusMetrics;

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

    pub blob_content_cache: BlobContentCache,
    pub data_readers: DataReadersList,

    pub multipart_list: MultipartList,
    pub persist_io: PersistIoOperations,
    pub init_state: InitState,
    pub settings: Arc<SettingsModel>,
    pub sync: EventsLoop<SyncEvent>,
    pub states: Arc<AppStates>,
    pub persist_markers: PersistMarkersByTable,
    persist_amount: AtomicUsize,
}

impl AppContext {
    pub fn new(
        logs: Arc<Logs>,
        settings: Arc<SettingsModel>,
        persist_io: PersistIoOperations,
    ) -> Self {
        AppContext {
            persist_markers: PersistMarkersByTable::new(),
            created: DateTimeAsMicroseconds::now(),
            init_state: InitState::new(),
            db: DbInstance::new(),
            logs,
            metrics: PrometheusMetrics::new(),
            active_transactions: ActiveTransactions::new(),
            process_id: uuid::Uuid::new_v4().to_string(),
            states: Arc::new(AppStates::create_un_initialized()),

            blob_content_cache: BlobContentCache::new(),
            data_readers: DataReadersList::new(Duration::from_secs(30)),
            multipart_list: MultipartList::new(),
            persist_io,
            settings,
            persist_amount: AtomicUsize::new(0),
            sync: EventsLoop::new("SyncEventsLoop".to_string()),
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

impl Logger for AppContext {
    fn write_info(
        &self,
        process_name: String,
        message: String,
        context: Option<HashMap<String, String>>,
    ) {
        self.logs
            .add_info(None, SystemProcess::System, process_name, message, context);
    }

    fn write_error(
        &self,
        process_name: String,
        message: String,
        context: Option<HashMap<String, String>>,
    ) {
        self.logs
            .add_fatal_error(None, SystemProcess::System, process_name, message, context);
    }

    fn write_warning(
        &self,
        process_name: String,
        message: String,
        ctx: Option<HashMap<String, String>>,
    ) {
        self.logs
            .add_error(None, SystemProcess::System, process_name, message, ctx);
    }

    fn write_fatal_error(
        &self,
        process_name: String,
        message: String,
        ctx: Option<HashMap<String, String>>,
    ) {
        self.logs
            .add_error(None, SystemProcess::System, process_name, message, ctx);
    }

    fn write_debug_info(
        &self,
        process_name: String,
        message: String,
        ctx: Option<HashMap<String, String>>,
    ) {
        self.logs
            .add_error(None, SystemProcess::Debug, process_name, message, ctx);
    }
}
