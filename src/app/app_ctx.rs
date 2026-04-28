use std::{
    sync::{atomic::AtomicUsize, Arc},
    time::Duration,
};

use my_no_sql_sdk::core::rust_extensions::{
    AppStates, date_time::DateTimeAsMicroseconds, events_loop::EventsLoop, file_utils::FilePath
};
use my_no_sql_sdk::server::DbInstance;

use crate::{
    data_readers::DataReadersList, db_operations::multipart::MultipartList, db_sync::SyncEvent, db_transactions::ActiveTransactions, operations::init::InitState, persist_markers::PersistMarkers, settings_reader::SettingsModel
};

use super::{HttpWriters, PrometheusMetrics};

pub const APP_VERSION: &'static str = env!("CARGO_PKG_VERSION");

pub const APP_NAME: &'static str = env!("CARGO_PKG_NAME");

pub const DEFAULT_PERSIST_PERIOD: crate::db_sync::DataSynchronizationPeriod =
    crate::db_sync::DataSynchronizationPeriod::Sec5;

pub struct AppContext {
    pub created: DateTimeAsMicroseconds,
    pub db: DbInstance,

    pub metrics: PrometheusMetrics,

    pub active_transactions: ActiveTransactions,

    pub data_readers: DataReadersList,

    pub multipart_list: MultipartList,
    //pub persist_io: PersistIoOperations,
    pub init_state: InitState,
    pub repo: crate::sqlite_repo::SqlLiteRepo,

    pub settings: Arc<SettingsModel>,
    pub sync: EventsLoop<SyncEvent>,
    pub states: Arc<AppStates>,
    pub persist_markers: PersistMarkers,
    pub http_writers: HttpWriters,
    persist_amount: AtomicUsize,

    pub use_unix_socket: Option<FilePath>,
}

impl AppContext {
    pub async fn new(settings: Arc<SettingsModel>) -> Self {
        Self {
            persist_markers: PersistMarkers::new(),
            created: DateTimeAsMicroseconds::now(),
            db: DbInstance::new(),
            metrics: PrometheusMetrics::new(),
            active_transactions: ActiveTransactions::new(),
            states: Arc::new(AppStates::create_un_initialized()),

            data_readers: DataReadersList::new(Duration::from_secs(30)),
            multipart_list: MultipartList::new(),
            repo: settings.get_sqlite_repo().await,
            settings,
            persist_amount: AtomicUsize::new(0),
            sync: EventsLoop::new("Sync"),
            http_writers: HttpWriters::new(),
            init_state: InitState::new(),
            use_unix_socket: match std::env::var("UNIX_SOCKET") {
                Ok(path) => FilePath::from_str(&path).into(),
                Err(_) => None,
            },
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
