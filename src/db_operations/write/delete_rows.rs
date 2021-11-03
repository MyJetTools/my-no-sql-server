use std::sync::Arc;

use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    app::AppContext,
    db::DbTable,
    db_sync::{states::DeleteEventSyncData, SyncAttributes, SyncEvent},
};

pub async fn execute(
    app: &AppContext,
    db_table: Arc<DbTable>,
    partition_key: &str,
    row_keys: Vec<String>,
    attr: Option<SyncAttributes>,
) {
    let mut table_write_access = db_table.data.write().await;

    let now = DateTimeAsMicroseconds::now();

    let db_partition = table_write_access.partitions.get_mut(partition_key);

    if db_partition.is_none() {
        return;
    }

    let db_partition = db_partition.unwrap();

    let mut sync = if let Some(attr) = attr {
        Some(DeleteEventSyncData::new(db_table.as_ref(), attr))
    } else {
        None
    };

    for row_key in row_keys {
        let delete_row_result = db_partition.remove(row_key.as_str(), now);

        if let Some(deleted_row) = delete_row_result {
            if let Some(state) = &mut sync {
                state.add_deleted_row(partition_key, row_key, deleted_row);
            }
        }
    }

    if let Some(state) = sync {
        if state.has_records() {
            app.events_dispatcher
                .dispatch(SyncEvent::Delete(state))
                .await
        }
    }
}
