use std::sync::Arc;

use crate::{
    app::AppContext,
    db::DbTable,
    db_sync::{states::InitTableEventSyncData, SyncAttributes, SyncEvent},
};

pub async fn execute(app: &AppContext, db_table: Arc<DbTable>, attr: Option<SyncAttributes>) {
    let mut table_data = db_table.data.write().await;

    let removed_partitions = table_data.clean_table();

    if removed_partitions.is_some() {
        if let Some(attr) = attr {
            let sync_data = InitTableEventSyncData::new(&table_data, attr);

            app.events_dispatcher
                .dispatch(SyncEvent::InitTable(sync_data))
                .await
        }
    }
}
