use std::sync::Arc;

use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    app::AppContext,
    db::DbTableWrapper,
    db_operations::DbOperationError,
    db_sync::{states::InitTableEventSyncData, EventSource, SyncEvent},
};

pub async fn execute(
    app: &AppContext,
    db_table_wrapper: Arc<DbTableWrapper>,
    event_src: EventSource,
    persist_moment: DateTimeAsMicroseconds,
) -> Result<(), DbOperationError> {
    super::super::check_app_states(app)?;
    let mut write_access = db_table_wrapper.data.write().await;

    let removed_partitions = write_access.db_table.clean_table();

    if removed_partitions.is_some() {
        let sync_data = InitTableEventSyncData::new(db_table_wrapper.clone(), event_src);

        crate::operations::sync::dispatch(app, SyncEvent::InitTable(sync_data));

        write_access
            .persist_markers
            .data_to_persist
            .mark_table_to_persist(persist_moment);
    }

    Ok(())
}
