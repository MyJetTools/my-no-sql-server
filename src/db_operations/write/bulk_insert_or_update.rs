use std::{collections::BTreeMap, sync::Arc};

use crate::{
    app::AppContext,
    db::{DbRow, DbTable},
    db_json_entity::JsonTimeStamp,
    db_sync::{states::UpdateRowsSyncData, EventSource, SyncEvent},
};

pub async fn execute(
    app: &AppContext,
    db_table: Arc<DbTable>,
    rows_by_partition: BTreeMap<String, Vec<Arc<DbRow>>>,
    event_src: Option<EventSource>,
    now: &JsonTimeStamp,
) {
    let mut table_data = db_table.data.write().await;

    let mut update_rows_state = if let Some(event_src) = event_src {
        Some(UpdateRowsSyncData::new(
            &table_data,
            db_table.attributes.get_persist(),
            event_src,
        ))
    } else {
        None
    };

    for (partition_key, db_rows) in rows_by_partition {
        table_data.bulk_insert_or_replace(&partition_key, &db_rows, now);

        if let Some(state) = &mut update_rows_state {
            state.add_rows(partition_key.as_str(), db_rows);
        }
    }

    if let Some(state) = update_rows_state {
        app.events_dispatcher
            .dispatch(SyncEvent::UpdateRows(state))
            .await
    }
}
