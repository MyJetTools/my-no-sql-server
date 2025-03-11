use std::{
    sync::{atomic::AtomicUsize, Arc},
    time::Duration,
};

use my_no_sql_sdk::core::rust_extensions::{date_time::DateTimeAsMicroseconds, AppStates};
use my_no_sql_sdk::server::DbInstance;

use crate::{
    data_readers::DataReadersList, db_operations::multipart::MultipartList,
    db_transactions::ActiveTransactions, operations::init::InitState,
    persist_markers::PersistMarkers, settings_reader::SettingsModel,
};

use super::{EventsSync, HttpWriters, PrometheusMetrics};

pub const APP_VERSION: &'static str = env!("CARGO_PKG_VERSION");

pub const DEFAULT_PERSIST_PERIOD: crate::db_sync::DataSynchronizationPeriod =
    crate::db_sync::DataSynchronizationPeriod::Sec5;

pub struct AppContext {
    pub created: DateTimeAsMicroseconds,
    pub db: DbInstance,

    pub metrics: PrometheusMetrics,

    pub active_transactions: ActiveTransactions,
    pub process_id: String,

    pub data_readers: DataReadersList,

    pub multipart_list: MultipartList,
    //pub persist_io: PersistIoOperations,
    pub init_state: InitState,
    pub repo: crate::sqlite_repo::SqlLiteRepo,

    pub settings: Arc<SettingsModel>,
    pub sync: EventsSync,
    pub states: Arc<AppStates>,
    pub persist_markers: PersistMarkers,
    pub http_writers: HttpWriters,
    persist_amount: AtomicUsize,
}

impl AppContext {
    pub async fn new(settings: Arc<SettingsModel>) -> Self {
        AppContext {
            persist_markers: PersistMarkers::new(),
            created: DateTimeAsMicroseconds::now(),
            db: DbInstance::new(),
            metrics: PrometheusMetrics::new(),
            active_transactions: ActiveTransactions::new(),
            process_id: uuid::Uuid::new_v4().to_string(),
            states: Arc::new(AppStates::create_un_initialized()),

            data_readers: DataReadersList::new(Duration::from_secs(30)),
            multipart_list: MultipartList::new(),
            repo: settings.get_sqlite_repo().await,
            settings,
            persist_amount: AtomicUsize::new(0),
            sync: EventsSync::new(),
            http_writers: HttpWriters::new(),
            init_state: InitState::new(),
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
