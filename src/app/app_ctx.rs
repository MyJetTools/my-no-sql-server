use std::{
    sync::{atomic::AtomicUsize, Arc},
    time::Duration,
};

use rust_extensions::{
    date_time::DateTimeAsMicroseconds, events_loop::EventsLoop, AppStates, Logger,
};

use crate::{
    data_readers::DataReadersList, db::DbInstance, db_operations::multipart::MultipartList,
    db_sync::SyncEvent, db_transactions::ActiveTransactions,
    persist_grpc_service::PersistGrpcService, settings_reader::SettingsModel,
};

use super::{
    logs::{Logs, SystemProcess},
    PrometheusMetrics,
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

    pub data_readers: DataReadersList,

    pub multipart_list: MultipartList,
    pub settings: SettingsModel,
    pub sync: EventsLoop<SyncEvent>,
    pub states: Arc<AppStates>,

    pub persist_grpc_service: PersistGrpcService,
    pub persist_retry_timeout: Duration,
    persist_amount: AtomicUsize,
}

impl AppContext {
    pub async fn new(logs: Arc<Logs>, settings: SettingsModel) -> Self {
        let persist_grpc_service = PersistGrpcService::new(settings.persistence_dest.clone()).await;
        AppContext {
            persist_retry_timeout: Duration::from_secs(5),
            created: DateTimeAsMicroseconds::now(),
            db: DbInstance::new(),
            logs,
            metrics: PrometheusMetrics::new(),
            active_transactions: ActiveTransactions::new(),
            process_id: uuid::Uuid::new_v4().to_string(),
            states: Arc::new(AppStates::create_un_initialized()),

            data_readers: DataReadersList::new(Duration::from_secs(30)),
            multipart_list: MultipartList::new(),
            settings,
            persist_amount: AtomicUsize::new(0),
            sync: EventsLoop::new("SyncEventsLoop".to_string()),
            persist_grpc_service,
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
    fn write_info(&self, process_name: String, message: String, context: Option<String>) {
        self.logs
            .add_info(None, SystemProcess::System, process_name, message, context);
    }

    fn write_error(&self, process_name: String, message: String, context: Option<String>) {
        self.logs
            .add_fatal_error(None, SystemProcess::System, process_name, message, context);
    }

    fn write_warning(&self, process_name: String, message: String, ctx: Option<String>) {
        self.logs
            .add_error(None, SystemProcess::System, process_name, message, ctx);
    }

    fn write_fatal_error(&self, process_name: String, message: String, ctx: Option<String>) {
        self.logs
            .add_error(None, SystemProcess::System, process_name, message, ctx);
    }
}
