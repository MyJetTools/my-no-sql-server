use std::{
    sync::{
        atomic::{AtomicI64, AtomicUsize},
        Arc,
    },
    time::Duration,
};

use my_no_sql_sdk::core::rust_extensions::{
    AppStates, date_time::DateTimeAsMicroseconds, events_loop::EventsLoop, file_utils::FilePath
};
use my_no_sql_sdk::server::DbInstance;

use crate::{
    data_readers::DataReadersList, db_operations::multipart::MultipartList, db_sync::SyncEvent, db_transactions::ActiveTransactions, operations::init::InitState, persist_markers::PersistMarkers, settings_reader::SettingsModel
};

use super::{HttpWriters, OneSecondCounter, PrometheusMetrics, WritersTraffic};

pub const APP_VERSION: &'static str = env!("CARGO_PKG_VERSION");

pub const APP_NAME: &'static str = env!("CARGO_PKG_NAME");

pub const DEFAULT_PERSIST_PERIOD: crate::db_sync::DataSynchronizationPeriod =
    crate::db_sync::DataSynchronizationPeriod::Sec5;

/// How long MCP write operations stay enabled after the user clicks
/// "Enable MCP writes" in the UI Settings page.
pub const MCP_WRITES_WINDOW: Duration = Duration::from_secs(600);

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

    pub write_payloads_per_second: OneSecondCounter,
    pub write_bytes_per_second: OneSecondCounter,
    pub writers_traffic: WritersTraffic,

    pub use_unix_socket: Option<FilePath>,

    /// Expiry (`unix_microseconds`) of the current MCP-writes enable
    /// window. `0` means MCP writes are disabled. Runtime-only — never
    /// persisted, so a restart always leaves MCP writes disabled.
    mcp_writes_enabled_until: AtomicI64,
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
            write_payloads_per_second: OneSecondCounter::new(),
            write_bytes_per_second: OneSecondCounter::new(),
            writers_traffic: WritersTraffic::new(),
            init_state: InitState::new(),
            use_unix_socket: match std::env::var("UNIX_SOCKET") {
                Ok(path) => FilePath::from_str(&path).into(),
                Err(_) => None,
            },
            mcp_writes_enabled_until: AtomicI64::new(0),
        }
    }

    /// Enables MCP write operations for `MCP_WRITES_WINDOW`.
    pub fn enable_mcp_writes(&self) {
        let until = DateTimeAsMicroseconds::now().add(MCP_WRITES_WINDOW);
        self.mcp_writes_enabled_until
            .store(until.unix_microseconds, std::sync::atomic::Ordering::SeqCst);
    }

    /// Disables MCP write operations immediately.
    pub fn disable_mcp_writes(&self) {
        self.mcp_writes_enabled_until
            .store(0, std::sync::atomic::Ordering::SeqCst);
    }

    /// `Some(expiry)` while the enable window is still open, otherwise `None`.
    pub fn mcp_writes_enabled_until(&self) -> Option<DateTimeAsMicroseconds> {
        let until = self
            .mcp_writes_enabled_until
            .load(std::sync::atomic::Ordering::Relaxed);
        if until > DateTimeAsMicroseconds::now().unix_microseconds {
            Some(DateTimeAsMicroseconds::new(until))
        } else {
            None
        }
    }

    pub fn is_mcp_write_enabled(&self) -> bool {
        self.mcp_writes_enabled_until().is_some()
    }

    /// Seconds left in the enable window, or `None` when disabled.
    pub fn mcp_writes_remaining_secs(&self) -> Option<u64> {
        let until = self.mcp_writes_enabled_until()?;
        let now = DateTimeAsMicroseconds::now();
        let micros_left = until.unix_microseconds - now.unix_microseconds;
        Some((micros_left.max(0) / 1_000_000) as u64)
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
