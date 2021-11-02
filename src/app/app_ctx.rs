use std::sync::Arc;

use tokio::sync::mpsc::UnboundedSender;

use crate::{
    db::DbInstance,
    db_transactions::ActiveTransactions,
    persistence::{
        blob_content_cache::BlobContentCache, updates_to_persist::UpdatesToPersistByTable,
    },
    settings_reader::SettingsModel,
    tcp::SessionsList,
};

use super::{
    global_states::GlobalStates, logs::Logs, metrics::PrometheusMetrics, EventsDispatcher,
};

pub const APP_VERSION: &'static str = env!("CARGO_PKG_VERSION");

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

    pub events_dispatcher: EventsDispatcher,
    pub blob_content_cache: BlobContentCache,
    pub data_readers: SessionsList,

    pub updates_to_persist_by_table: UpdatesToPersistByTable,
}

impl AppContext {
    pub fn new(settings: &SettingsModel, sender: Option<UnboundedSender<()>>) -> Self {
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
            events_dispatcher: EventsDispatcher::new(sender),
            blob_content_cache: BlobContentCache::new(),
            data_readers: SessionsList::new(),
            updates_to_persist_by_table: UpdatesToPersistByTable::new(),
        }
    }
}
