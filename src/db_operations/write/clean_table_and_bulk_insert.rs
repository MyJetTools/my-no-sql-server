use std::{collections::BTreeMap, sync::Arc};

use crate::{
    app::AppContext,
    db::{DbRow, DbTable},
    db_json_entity::JsonTimeStamp,
    db_operations::DbOperationError,
    db_sync::{states::InitTableEventSyncData, SyncAttributes, SyncEvent},
};

pub async fn execute(
    app: &AppContext,
    db_table: Arc<DbTable>,
    entities: BTreeMap<String, Vec<Arc<DbRow>>>,
    attr: Option<SyncAttributes>,
    now: &JsonTimeStamp,
) -> Result<(), DbOperationError> {
    let mut table_data = db_table.data.write().await;

    table_data.clean_table();

    for (partition_key, db_rows) in entities {
        table_data.bulk_insert_or_replace(partition_key.as_str(), &db_rows, now);
    }

    if let Some(attr) = attr {
        let sync_data = InitTableEventSyncData::new(&table_data, attr);

        app.events_dispatcher
            .dispatch(SyncEvent::InitTable(sync_data))
            .await;
    }

    Ok(())
}
