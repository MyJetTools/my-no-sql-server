use std::sync::Arc;

use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    app::AppContext,
    db::{DbRow, DbTable},
    db_sync::{states::DeleteRowsEventSyncData, SyncAttributes, SyncEvent},
};

pub async fn execute(
    app: &AppContext,
    db_table: Arc<DbTable>,
    partition_key: &str,
    row_key: &str,
    attr: Option<SyncAttributes>,
    now: DateTimeAsMicroseconds,
) -> Option<Arc<DbRow>> {
    let mut table_write_access = db_table.data.write().await;

    let remove_row_result = {
        let db_partition = table_write_access.partitions.get_mut(partition_key);

        if db_partition.is_none() {
            return None;
        }

        let db_partition = db_partition.unwrap();

        let remove_result = super::db_actions::remove_db_row(
            app,
            db_table.name.as_str(),
            db_partition,
            row_key,
            now,
        )
        .await;

        if remove_result.is_none() {
            return None;
        }

        remove_result.unwrap()
    };

    let mut sync_data = if let Some(attr) = attr {
        let mut sync_data = DeleteRowsEventSyncData::new(db_table.as_ref(), attr);
        sync_data.add_deleted_row(partition_key, remove_row_result.removed_row.clone());
        Some(sync_data)
    } else {
        None
    };

    super::db_actions::handle_after_delete_row(
        &mut table_write_access,
        partition_key,
        &remove_row_result,
        sync_data.as_mut(),
    );

    if let Some(sync_data) = sync_data {
        app.events_dispatcher
            .dispatch(SyncEvent::DeleteRows(sync_data))
            .await
    }

    return Some(remove_row_result.removed_row);
}
