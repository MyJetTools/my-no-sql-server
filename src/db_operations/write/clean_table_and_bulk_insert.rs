use std::{collections::BTreeMap, sync::Arc};

use my_no_sql_core::{
    db::{DbRow, DbTable},
    db_json_entity::JsonTimeStamp,
};
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    app::AppContext,
    db_operations::DbOperationError,
    db_sync::{states::InitTableEventSyncData, EventSource, SyncEvent},
};

pub async fn execute(
    app: &AppContext,
    db_table: Arc<DbTable>,
    entities: BTreeMap<String, Vec<Arc<DbRow>>>,
    event_src: Option<EventSource>,
    now: &JsonTimeStamp,
    persist_moment: DateTimeAsMicroseconds,
) -> Result<(), DbOperationError> {
    super::super::check_app_states(app)?;
    let mut table_data = db_table.data.write().await;

    table_data.clean_table();

    for (partition_key, db_rows) in entities {
        table_data.bulk_insert_or_replace(partition_key.as_str(), &db_rows, now);
    }

    app.persist_markers
        .persist_table(table_data.name.as_str(), persist_moment)
        .await;

    if let Some(event_src) = event_src {
        let sync_data =
            InitTableEventSyncData::new(&table_data, db_table.attributes.get_snapshot(), event_src);

        crate::operations::sync::dispatch(app, SyncEvent::InitTable(sync_data));
    }

    Ok(())
}
