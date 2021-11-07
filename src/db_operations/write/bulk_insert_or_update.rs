use std::{collections::BTreeMap, sync::Arc};

use crate::{
    app::AppContext,
    db::{DbRow, DbTable},
    db_json_entity::JsonTimeStamp,
    db_sync::{states::UpdateRowsSyncData, SyncAttributes, SyncEvent},
};

pub async fn execute(
    app: &AppContext,
    db_table: Arc<DbTable>,
    rows_by_partition: BTreeMap<String, Vec<Arc<DbRow>>>,
    attr: Option<SyncAttributes>,
    now: &JsonTimeStamp,
) {
    let mut table_write_access = db_table.data.write().await;

    let mut update_rows_state = if let Some(attr) = attr {
        Some(UpdateRowsSyncData::new(db_table.as_ref(), attr))
    } else {
        None
    };

    for (partition_key, db_rows) in rows_by_partition {
        let db_partition = table_write_access.get_or_create_partition(partition_key.as_str());

        super::db_actions::bulk_remove_db_rows(
            app,
            db_table.name.as_str(),
            db_partition,
            db_rows.iter().map(|itm| &itm.row_key),
            now,
        )
        .await;

        super::db_actions::bulk_insert_db_rows(
            app,
            db_table.name.as_str(),
            db_partition,
            db_rows.as_slice(),
            now,
        )
        .await;

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
