use std::{collections::BTreeMap, sync::Arc};

use crate::{
    app::AppContext,
    db::{DbRow, DbTable, DbTableSnapshot},
    db_operations::DbOperationError,
    db_sync::{states::InitTableEventSyncData, SyncAttributes, SyncEvent},
};

pub async fn execute(
    app: &AppContext,
    db_table: Arc<DbTable>,
    entities: BTreeMap<String, Vec<Arc<DbRow>>>,
    attr: Option<SyncAttributes>,
) -> Result<(), DbOperationError> {
    let mut table_write_access = db_table.data.write().await;

    if table_write_access.partitions.len() == 0 {
        return Ok(());
    }

    super::db_actions::clean_table(app, &mut table_write_access).await;

    let sync = if let Some(attr) = attr {
        Some(InitTableEventSyncData::new(db_table.as_ref(), attr))
    } else {
        table_write_access.partitions.clear();
        None
    };

    for (partition_key, db_rows) in entities {
        let db_partition = table_write_access.get_or_create_partition(partition_key.as_str());

        let now = db_rows[0].time_stamp;
        super::db_actions::bulk_insert_db_rows(
            app,
            db_table.name.as_str(),
            db_partition,
            &db_rows,
            now,
        )
        .await;
    }

    if let Some(mut state) = sync {
        let table_snapshot = DbTableSnapshot::new(&table_write_access);

        state.add_table_snapshot(table_snapshot);
        app.events_dispatcher
            .dispatch(SyncEvent::InitTable(state))
            .await;
    }

    Ok(())
}
