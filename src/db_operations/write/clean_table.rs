use std::sync::Arc;

use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    app::AppContext,
    db::DbTable,
    db_operations::DbOperationError,
    db_sync::{states::InitTableEventSyncData, EventSource, SyncEvent},
};

pub async fn execute(
    app: &AppContext,
    db_table: Arc<DbTable>,
    event_src: EventSource,
    persist_moment: DateTimeAsMicroseconds,
) -> Result<(), DbOperationError> {
    super::super::check_app_states(app)?;
    let mut table_data = db_table.data.write().await;

    let removed_partitions = table_data.clean_table();

    if removed_partitions.is_some() {
        let sync_data = InitTableEventSyncData::new(
            db_table.as_ref(),
            &table_data,
            db_table.attributes.get_snapshot(),
            event_src,
        );

        crate::operations::sync::dispatch(
            app,
            db_table.as_ref().into(),
            SyncEvent::InitTable(sync_data),
        );

        table_data
            .data_to_persist
            .mark_table_to_persist(persist_moment);
    }

    Ok(())
}
