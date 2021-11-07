use std::sync::Arc;

use crate::{
    app::AppContext,
    db::{DbRow, DbTable},
    db_json_entity::JsonTimeStamp,
    db_sync::{states::UpdateRowsSyncData, SyncAttributes, SyncEvent},
};

pub async fn execute(
    app: &AppContext,
    db_table: Arc<DbTable>,
    partition_key: &str,
    db_row: Arc<DbRow>,
    attr: Option<SyncAttributes>,
    now: &JsonTimeStamp,
) -> Option<Arc<DbRow>> {
    let mut table_write_access = db_table.data.write().await;

    let db_partition = table_write_access.get_or_create_partition(partition_key);

    let result = db_partition.insert_or_replace(db_row.clone(), Some(now));

    if let Some(attr) = attr {
        let mut update_rows_state = UpdateRowsSyncData::new(db_table.as_ref(), attr);

        update_rows_state.add_row(partition_key, db_row);
        app.events_dispatcher
            .dispatch(SyncEvent::UpdateRows(update_rows_state))
            .await;
    }

    result
}
