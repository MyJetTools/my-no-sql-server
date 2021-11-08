use crate::{
    app::AppContext,
    db_sync::{DataSynchronizationPeriod, SyncAttributes},
};

pub fn create(app: &AppContext, sync_period: DataSynchronizationPeriod) -> SyncAttributes {
    let locations = vec![app.location.to_string()];
    SyncAttributes {
        locations,
        event_source: crate::db_sync::EventSource::ClientRequest,
        headers: None, //TODO - Enable Headers,
        sync_period,
    }
}
