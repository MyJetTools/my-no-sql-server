use crate::db_sync::EventSource;

use super::SyncTableData;

pub struct UpdateTableAttributesSyncData {
    pub table_data: SyncTableData,
    pub event_src: EventSource,
}
