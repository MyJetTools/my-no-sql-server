use std::{
    sync::{atomic::AtomicUsize, Arc},
    time::Duration,
};

use my_no_sql_sdk::core::rust_extensions::{date_time::DateTimeAsMicroseconds, AppStates};
use my_no_sql_server_core::DbInstance;

use crate::{
    data_readers::DataReadersList,
    db_operations::multipart::MultipartList,
    db_transactions::ActiveTransactions,
    persist::PersistMarkersByTable,
    persist_io::PersistIoOperations,
    persist_operations::{
        blob_content_cache::BlobContentCache, data_initializer::load_tasks::InitState,
    },
    settings_reader::SettingsModel,
};

use super::{EventsSync, PrometheusMetrics};

pub const APP_VERSION: &'static str = env!("CARGO_PKG_VERSION");

pub const DEFAULT_PERSIST_PERIOD: crate::db_sync::DataSynchronizationPeriod =
    crate::db_sync::DataSynchronizationPeriod::Sec5;

pub struct AppContext {
    pub created: DateTimeAsMicroseconds,
    pub db: DbInstance,

    pub metrics: PrometheusMetrics,

    pub active_transactions: ActiveTransactions,
    pub process_id: String,

    pub blob_content_cache: BlobContentCache,
    pub data_readers: DataReadersList,

    pub multipart_list: MultipartList,
    pub persist_io: PersistIoOperations,
    pub init_state: InitState,
    pub settings: Arc<SettingsModel>,
    pub sync: EventsSync,
    pub states: Arc<AppStates>,
    pub persist_markers: PersistMarkersByTable,
    persist_amount: AtomicUsize,
}

impl AppContext {
    pub fn new(settings: Arc<SettingsModel>, persist_io: PersistIoOperations) -> Self {
        AppContext {
            persist_markers: PersistMarkersByTable::new(),
            created: DateTimeAsMicroseconds::now(),
            init_state: InitState::new(),
            db: DbInstance::new(),
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
            sync: EventsSync::new(),
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
