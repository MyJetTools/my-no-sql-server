use std::sync::Arc;

use my_no_sql_server_core::DbTableWrapper;
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    app::AppContext,
    db_operations::DbOperationError,
    db_sync::{states::InitTableEventSyncData, EventSource, SyncEvent},
};

pub async fn execute(
    app: &AppContext,
    db_table: &Arc<DbTableWrapper>,
    event_src: EventSource,
    persist_moment: DateTimeAsMicroseconds,
) -> Result<(), DbOperationError> {
    super::super::check_app_states(app)?;
    let mut table_data = db_table.data.write().await;

    let removed_partitions = table_data.clean_table();

    if removed_partitions.is_some() {
        let sync_data = InitTableEventSyncData::new(&table_data, event_src);

        crate::operations::sync::dispatch(app, SyncEvent::InitTable(sync_data));

        app.persist_markers
            .persist_table(table_data.name.as_str(), persist_moment)
            .await;
    }

    Ok(())
}
