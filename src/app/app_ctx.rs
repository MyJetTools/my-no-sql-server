use std::{sync::Arc, time::Duration};

use rust_extensions::ApplicationStates;

use crate::{
    data_readers::DataReadersList, db::DbInstance, db_operations::multipart::MultipartList,
    db_transactions::ActiveTransactions, persistence::blob_content_cache::BlobContentCache,
    settings_reader::SettingsModel,
};

use super::{global_states::GlobalStates, logs::Logs, EventsDispatcher, PrometheusMetrics};

pub const APP_VERSION: &'static str = env!("CARGO_PKG_VERSION");

pub const DEFAULT_PERSIST_PERIOD: crate::db_sync::DataSynchronizationPeriod =
    crate::db_sync::DataSynchronizationPeriod::Sec5;

pub struct AppContext {
    pub db: DbInstance,
    pub logs: Arc<Logs>,

    pub metrics: PrometheusMetrics,

    pub active_transactions: ActiveTransactions,
    pub process_id: String,

    pub states: GlobalStates,

    pub persist_to_blob: bool,

    pub location: String,

    pub compress_data: bool,

    pub table_api_key: String,

    pub events_dispatcher: Box<dyn EventsDispatcher + Send + Sync + 'static>,
    pub blob_content_cache: BlobContentCache,
    pub data_readers: DataReadersList,

    pub multipart_list: MultipartList,
}

impl AppContext {
    pub fn new(
        settings: &SettingsModel,
        events_dispatcher: Box<dyn EventsDispatcher + Send + Sync + 'static>,
    ) -> Self {
        AppContext {
            db: DbInstance::new(),
            persist_to_blob: settings.persist_to_blob(),
            logs: Arc::new(Logs::new()),
            metrics: PrometheusMetrics::new(),
            active_transactions: ActiveTransactions::new(),
            process_id: uuid::Uuid::new_v4().to_string(),
            states: GlobalStates::new(),

            location: settings.location.to_string(),
            compress_data: settings.compress_data,
            table_api_key: settings.table_api_key.to_string(),
            events_dispatcher,
            blob_content_cache: BlobContentCache::new(),
            data_readers: DataReadersList::new(Duration::from_secs(30)),
            multipart_list: MultipartList::new(),
        }
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
