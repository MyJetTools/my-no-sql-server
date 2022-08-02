use std::sync::Arc;

use crate::{db::DbTableWrapper, db_sync::EventSource};

pub struct InitTableEventSyncData {
    pub db_table: Arc<DbTableWrapper>,
    pub event_src: EventSource,
}

impl InitTableEventSyncData {
    pub fn new(db_table: Arc<DbTableWrapper>, event_src: EventSource) -> Self {
        Self {
            db_table,
            event_src,
        }
    }
}
