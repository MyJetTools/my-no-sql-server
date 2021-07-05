use crate::{
    app::AppServices,
    db_transactions::{DataSynchronizationPeriod, TransactionAttributes},
};

pub fn create_transaction_attributes(
    app: &AppServices,
    sync_period: DataSynchronizationPeriod,
) -> TransactionAttributes {
    let locations = vec![app.settings.location.to_string()];
    TransactionAttributes {
        locations,
        event_source: crate::db_transactions::EventSource::ClientRequest,
        headers: None, //TODO - Enable Headers,
        sync_period,
    }
}
