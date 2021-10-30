use std::{collections::BTreeMap, sync::Arc};

use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    app::AppContext,
    db::{DbRow, DbTable, DbTableSnapshot},
    db_operations::DbOperationError,
    db_sync::{states::InitTableEventState, SyncAttributes, SyncEvent},
};

pub async fn execute(
    app: &AppContext,
    db_table: Arc<DbTable>,
    entities: BTreeMap<String, Vec<Arc<DbRow>>>,
    attr: Option<SyncAttributes>,
) -> Result<(), DbOperationError> {
    let now = DateTimeAsMicroseconds::now();

    let mut table_write_access = db_table.data.write().await;

    if table_write_access.partitions.len() == 0 {
        return Ok(());
    }

    let sync = if let Some(attr) = attr {
        let mut init_state = InitTableEventState::new(db_table.clone(), attr);
        super::clean_table::clean_table(&mut table_write_access, &mut init_state);

        Some(init_state)
    } else {
        table_write_access.partitions.clear();
        None
    };

    for (partition_key, rows) in entities {
        let db_partition =
            table_write_access.get_or_create_partition(partition_key.as_str(), Some(now));

        db_partition.bulk_insert_or_replace(&rows, Some(now));
    }

    if let Some(mut state) = sync {
        let table_snapshot = DbTableSnapshot::new(&table_write_access, None);

        state.add_table_snapshot(table_snapshot);
        app.events_dispatcher
            .dispatch(SyncEvent::InitTable(state))
            .await;
    }

    Ok(())
}
