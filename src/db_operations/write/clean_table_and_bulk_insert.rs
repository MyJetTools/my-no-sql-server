use std::sync::Arc;

use my_no_sql_sdk::core::db::DbRow;
use my_no_sql_sdk::core::rust_extensions::date_time::DateTimeAsMicroseconds;
use my_no_sql_sdk::server::DbTable;

use crate::{
    app::AppContext,
    db_operations::DbOperationError,
    db_sync::{states::InitTableEventSyncData, EventSource, SyncEvent},
};

pub async fn execute(
    app: &AppContext,
    db_table: Arc<DbTable>,
    entities: Vec<(String, Vec<Arc<DbRow>>)>,
    event_src: Option<EventSource>,
    persist_moment: DateTimeAsMicroseconds,
    now: DateTimeAsMicroseconds,
) -> Result<(), DbOperationError> {
    super::super::check_app_states(app)?;
    let mut table_data = db_table.data.write().await;

    table_data.clear_table();

    for (partition_key, db_rows) in entities {
        table_data.bulk_insert_or_replace(&partition_key, &db_rows, Some(now));
    }

    app.persist_markers
        .persist_table_content(&table_data.name, persist_moment)
        .await;

    if let Some(event_src) = event_src {
        let sync_data = InitTableEventSyncData::new(&table_data, event_src);

        crate::operations::sync::dispatch(app, SyncEvent::InitTable(sync_data));
    }

    Ok(())
}
