use std::{collections::BTreeMap, sync::Arc};

use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    app::AppContext,
    db::{DbRow, DbTable},
    db_sync::{states::UpdateRowsSyncState, SyncAttributes, SyncEvent},
};

pub async fn execute(
    app: &AppContext,
    db_table: Arc<DbTable>,
    rows_by_partition: BTreeMap<String, Vec<Arc<DbRow>>>,
    attr: Option<SyncAttributes>,
) {
    let now = DateTimeAsMicroseconds::now();

    let mut table_write_access = db_table.data.write().await;

    let mut update_rows_state = if let Some(attr) = attr {
        Some(UpdateRowsSyncState::new(db_table.clone(), attr))
    } else {
        None
    };

    for (partition_key, db_rows) in rows_by_partition {
        let db_partition =
            table_write_access.get_or_create_partition(partition_key.as_str(), Some(now));

        db_partition.bulk_insert_or_replace(&db_rows, Some(now));

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
