use std::sync::Arc;

use tokio::sync::mpsc::UnboundedSender;

use crate::{
    db::DbInstance,
    db_transactions::ActiveTransactions,
    persistence::{blob_content_cache::BlobContentCache, QueueToPersist},
    settings_reader::SettingsModel,
    tcp::SessionsList,
};

use super::{
    global_states::GlobalStates, logs::Logs, metrics::PrometheusMetrics, EventsDispatcher,
};

pub const APP_VERSION: &'static str = env!("CARGO_PKG_VERSION");

pub struct AppContext {
    pub db: DbInstance,
    pub queue_to_persist: QueueToPersist,
    pub logs: Arc<Logs>,

    pub metrics: PrometheusMetrics,

    pub active_transactions: ActiveTransactions,
    pub process_id: String,

    pub states: GlobalStates,
    pub data_readers: SessionsList,

    pub persist_to_blob: bool,

    pub location: String,

    pub compress_data: bool,

    pub table_api_key: String,

    pub blob_content_cache: BlobContentCache,

    pub events_dispatcher: EventsDispatcher,
}

impl AppContext {
    pub fn new(settings: &SettingsModel, sender: Option<UnboundedSender<()>>) -> Self {
        AppContext {
            db: DbInstance::new(),
            persist_to_blob: settings.persist_to_blob(),
            queue_to_persist: QueueToPersist::new(),
            logs: Arc::new(Logs::new()),
            metrics: PrometheusMetrics::new(),
            active_transactions: ActiveTransactions::new(),
            process_id: uuid::Uuid::new_v4().to_string(),
            states: GlobalStates::new(),
            data_readers: SessionsList::new(),
            location: settings.location.to_string(),
            compress_data: settings.compress_data,
            table_api_key: settings.table_api_key.to_string(),
            blob_content_cache: BlobContentCache::new(),
            events_dispatcher: EventsDispatcher::new(sender),
        }
    }
}
