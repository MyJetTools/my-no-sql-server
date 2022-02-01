use std::sync::Arc;

use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    app::AppContext,
    db::DbTable,
    db_sync::{states::InitTableEventSyncData, EventSource, SyncEvent},
};

pub async fn execute(
    app: &AppContext,
    db_table: Arc<DbTable>,
    event_src: EventSource,
    persist_moment: DateTimeAsMicroseconds,
) {
    let mut table_data = db_table.data.write().await;

    let removed_partitions = table_data.clean_table();

    if removed_partitions.is_some() {
        let sync_data =
            InitTableEventSyncData::new(&table_data, db_table.attributes.get_snapshot(), event_src);

        app.events_dispatcher
            .dispatch(SyncEvent::InitTable(sync_data));

        table_data
            .data_to_persist
            .mark_table_to_persist(persist_moment);
    }
}
