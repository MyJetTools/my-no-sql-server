use crate::db_sync::SyncAttributes;

use super::SyncTableData;

pub struct UpdateTableAttributesSyncData {
    pub table_data: SyncTableData,
    pub attr: SyncAttributes,
    pub persist: bool,
    pub max_partitions_amount: Option<usize>,
}
