use std::sync::Arc;

use my_no_sql_sdk::core::rust_extensions::date_time::DateTimeAsMicroseconds;
use my_no_sql_sdk::server::DbTable;

use crate::{
    app::AppContext,
    db_operations::DbOperationError,
    db_sync::{states::InitTableEventSyncData, EventSource, SyncEvent},
};

pub async fn clean_table(
    app: &AppContext,
    db_table: &Arc<DbTable>,
    event_src: EventSource,
    persist_moment: DateTimeAsMicroseconds,
) -> Result<(), DbOperationError> {
    super::super::check_app_states(app)?;
    let mut table_data = db_table.data.write().await;

    let removed_partitions = table_data.clear_table();

    if removed_partitions.is_some() {
        let sync_data = InitTableEventSyncData::new(&table_data, event_src);

        crate::operations::sync::dispatch(app, SyncEvent::InitTable(sync_data));

        app.persist_markers
            .persist_table_content(&table_data.name, persist_moment)
            .await;
    }

    Ok(())
}
