use std::sync::Arc;

use crate::{
    app::AppContext,
    db::DbTable,
    db_sync::{states::DeleteEventState, SyncAttributes, SyncEvent},
};

pub async fn execute(
    app: &AppContext,
    db_table: Arc<DbTable>,
    partition_key: &str,
    max_rows_amount: usize,
    attr: Option<SyncAttributes>,
) {
    let mut write_access = db_table.data.write().await;

    let partition = write_access.get_partition_mut(partition_key, None);

    if partition.is_none() {
        return;
    }

    let partition = partition.unwrap();

    let gced_rows_result = partition.gc_rows(max_rows_amount);

    if let Some(gced_rows) = gced_rows_result {
        if let Some(attr) = attr {
            let mut state = DeleteEventState::new(db_table.clone(), attr);

            for (row_key, db_row) in gced_rows {
                state.add_deleted_row(partition_key, row_key, db_row);
            }

            app.events_dispatcher
                .dispatch(SyncEvent::Delete(state))
                .await;
        }
    }
}
