use crate::{
    app::AppContext,
    db::DbTable,
    db_sync::{states::DeleteRowsEventSyncData, SyncAttributes, SyncEvent},
};

pub async fn execute(
    app: &AppContext,
    db_table: &DbTable,
    partition_key: &str,
    max_rows_amount: usize,
    attr: Option<SyncAttributes>,
) {
    let mut write_access = db_table.data.write().await;

    let partition = write_access.partitions.get_mut(partition_key);

    if partition.is_none() {
        return;
    }

    let partition = partition.unwrap();

    let gced_rows_result = partition.gc_rows(max_rows_amount);

    if let Some(gced_rows) = gced_rows_result {
        if let Some(attr) = attr {
            let mut sync_data = DeleteRowsEventSyncData::new(db_table, attr);

            sync_data.add_deleted_rows(partition_key, &gced_rows);

            app.events_dispatcher
                .dispatch(SyncEvent::DeleteRows(sync_data))
                .await;
        }
    }
}
