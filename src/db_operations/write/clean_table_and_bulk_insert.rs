use std::{collections::BTreeMap, sync::Arc};

use my_no_sql_core::{db::DbRow, db_json_entity::JsonTimeStamp};
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
    entities: BTreeMap<String, Vec<Arc<DbRow>>>,
    event_src: Option<EventSource>,
    now: &JsonTimeStamp,
    persist_moment: DateTimeAsMicroseconds,
) -> Result<(), DbOperationError> {
    super::super::check_app_states(app)?;
    let mut write_access = db_table_wrapper.data.write().await;

    write_access.db_table.clean_table();

    for (partition_key, db_rows) in entities {
        write_access
            .db_table
            .bulk_insert_or_replace(partition_key.as_str(), &db_rows, now);
    }

    write_access
        .persist_markers
        .data_to_persist
        .mark_table_to_persist(persist_moment);

    if let Some(event_src) = event_src {
        let sync_data = InitTableEventSyncData::new(db_table_wrapper.clone(), event_src);

        crate::operations::sync::dispatch(app, SyncEvent::InitTable(sync_data));
    }

    Ok(())
}
