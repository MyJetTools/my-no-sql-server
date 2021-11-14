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
    db_row: Arc<DbRow>,
    attr: Option<SyncAttributes>,
    now: &JsonTimeStamp,
) -> Option<Arc<DbRow>> {
    let mut table_data = db_table.data.write().await;

    let result = table_data.insert_or_replace_row(&db_row, now);

    if let Some(attr) = attr {
        let mut update_rows_state = UpdateRowsSyncData::new(&table_data, attr);

        update_rows_state.add_row(db_row);
        app.events_dispatcher
            .dispatch(SyncEvent::UpdateRows(update_rows_state))
            .await;
    }

    result
}
