use std::{collections::HashMap, sync::Arc};

use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    app::AppContext,
    db::DbTable,
    db_sync::{states::DeleteRowsEventSyncData, SyncAttributes, SyncEvent},
};

pub async fn execute(
    app: &AppContext,
    db_table: Arc<DbTable>,
    rows_to_delete: HashMap<String, Vec<String>>,
    attr: Option<SyncAttributes>,
) {
    let mut write_access = db_table.data.write().await;

    let now = DateTimeAsMicroseconds::now();

    let mut delete_event_state = if let Some(attr) = attr {
        Some(DeleteRowsEventSyncData::new(db_table.as_ref(), attr))
    } else {
        None
    };

    for (partition_key, row_keys) in rows_to_delete {
        let partition = write_access.partitions.get_mut(partition_key.as_str());

        if partition.is_none() {
            continue;
        }

        let partition = partition.unwrap();

        let remove_result = super::db_actions::bulk_remove_db_rows(
            app,
            db_table.name.as_str(),
            partition,
            row_keys.iter(),
            now,
        )
        .await;

        if let Some(delete_event_state) = &mut delete_event_state {
            if let Some(removed_rows) = remove_result {
                delete_event_state.add_deleted_rows(partition_key.as_str(), &removed_rows);
            }
        }

        if partition.rows_count() == 0 {
            let deleted_partition = write_access.partitions.remove(partition_key.as_str());

            if let Some(deleted_partition) = deleted_partition {
                if let Some(delete_event_state) = &mut delete_event_state {
                    delete_event_state.new_deleted_partition(partition_key, deleted_partition)
                }
            }
        }
    }

    if let Some(delete_event_state) = delete_event_state {
        app.events_dispatcher
            .dispatch(SyncEvent::Delete(delete_event_state))
            .await;
    }
}
