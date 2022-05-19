use std::{collections::BTreeMap, sync::Arc};

use crate::{
    app::AppContext,
    db::{DbRow, DbTable},
    db_json_entity::JsonTimeStamp,
    db_operations::DbOperationError,
    db_sync::{states::InitTableEventSyncData, EventSource, SyncEvent},
};

pub async fn execute(
    app: &AppContext,
    db_table: Arc<DbTable>,
    entities: BTreeMap<String, Vec<Arc<DbRow>>>,
    event_src: Option<EventSource>,
    now: &JsonTimeStamp,
) -> Result<(), DbOperationError> {
    super::super::check_app_states(app)?;
    let mut table_data = db_table.data.write().await;

    table_data.clean_table();

    for (partition_key, db_rows) in entities {
        table_data.bulk_insert_or_replace(partition_key.as_str(), &db_rows, now);
    }

    if let Some(event_src) = event_src {
        let sync_data =
            InitTableEventSyncData::new(&table_data, db_table.attributes.get_snapshot(), event_src);

        app.events_dispatcher
            .dispatch(SyncEvent::InitTable(sync_data));
    }

    Ok(())
}
