use std::{collections::BTreeMap, sync::Arc};

use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    app::AppContext,
    db::{DbRow, DbTable},
    db_operations::DbOperationError,
    db_sync::{states::UpdatePartitionsState, SyncAttributes, SyncEvent},
};

pub async fn execute(
    app: &AppContext,
    db_table: Arc<DbTable>,
    partition_key: &str,
    entities: BTreeMap<String, Vec<Arc<DbRow>>>,
    attr: Option<SyncAttributes>,
) -> Result<(), DbOperationError> {
    let now = DateTimeAsMicroseconds::now();

    let mut write_access = db_table.data.write().await;

    let mut update_partitions_state = if let Some(attr) = attr {
        Some(UpdatePartitionsState::new(db_table.clone(), attr))
    } else {
        None
    };

    let removed_partition_result = write_access.partitions.remove(partition_key);

    for (partition_key, db_rows) in entities {
        let db_partition = write_access.get_or_create_partition(partition_key.as_str());
        db_partition.last_write_moment.update(now);
        db_partition.bulk_insert_or_replace(&db_rows, Some(now));

        if let Some(state) = &mut update_partitions_state {
            state.add(
                partition_key,
                Some(db_partition.get_db_partition_snapshot()),
            );
        }
    }

    //TODO - Unit test usecase where deleted partition had not Rows to insert
    if removed_partition_result.is_some() {
        if let Some(update_partitions_state) = &mut update_partitions_state {
            if !update_partitions_state
                .partitions_to_update
                .contains_key(partition_key)
            {
                match write_access.partitions.get(partition_key) {
                    Some(partition) => update_partitions_state.add(
                        partition_key.to_string(),
                        Some(partition.get_db_partition_snapshot()),
                    ),
                    None => update_partitions_state.add(partition_key.to_string(), None),
                };
            }
        }
    }

    if let Some(state) = update_partitions_state {
        app.events_dispatcher
            .dispatch(SyncEvent::InitPartitions(state))
            .await;
    }

    Ok(())
}
