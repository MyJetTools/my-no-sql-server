use std::{sync::Arc, time::Duration};

use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    app::NextEventsToHandle, db::DbTable, db_sync::SyncEvent,
    persistence::updates_to_persist::UpdatesToPersistByTable,
};

const FIVE_SECS: i64 = Duration::from_secs(5).as_micros() as i64;

pub async fn execute(
    db_table: Option<Arc<DbTable>>,
    updates_to_persist_by_table: &UpdatesToPersistByTable,
    next_events: &NextEventsToHandle,
) {
    let mut sync_moment = DateTimeAsMicroseconds::now();

    sync_moment.unix_microseconds += FIVE_SECS; //TODO - Hardcoded 5 seconds persistence time

    if db_table.is_some() {
        for next_event in &next_events.events {
            match next_event {
                SyncEvent::UpdateTableAttributes(_) => {
                    updates_to_persist_by_table
                        .update_table_attributes(next_events.table_name.as_str(), sync_moment)
                        .await;
                }
                SyncEvent::InitTable(data) => {
                    if data.table_data.persist {
                        updates_to_persist_by_table
                            .flag_table_to_update(next_events.table_name.as_str(), sync_moment)
                            .await;
                    }
                }
                SyncEvent::InitPartitions(data) => {
                    if data.table_data.persist {
                        updates_to_persist_by_table
                            .flag_paritions_to_update(
                                next_events.table_name.as_str(),
                                data.partitions_to_update.keys(),
                                sync_moment,
                            )
                            .await;
                    }
                }
                SyncEvent::UpdateRows(data) => {
                    if data.table_data.persist {
                        updates_to_persist_by_table
                            .flag_paritions_to_update(
                                next_events.table_name.as_str(),
                                data.updated_rows_by_partition.keys(),
                                sync_moment,
                            )
                            .await;
                    }
                }
                SyncEvent::DeleteRows(data) => {
                    if data.table_data.persist {
                        if let Some(deleted_partitions) = &data.deleted_partitions {
                            updates_to_persist_by_table
                                .flag_paritions_to_update(
                                    next_events.table_name.as_str(),
                                    deleted_partitions.keys(),
                                    sync_moment,
                                )
                                .await;
                        }
                        if let Some(deleted_rows) = &data.deleted_rows {
                            updates_to_persist_by_table
                                .flag_paritions_to_update(
                                    next_events.table_name.as_str(),
                                    deleted_rows.keys(),
                                    sync_moment,
                                )
                                .await;
                        }
                    }
                }
                SyncEvent::DeleteTable(_) => {
                    updates_to_persist_by_table
                        .flag_table_to_update(next_events.table_name.as_str(), sync_moment)
                        .await;
                }
                SyncEvent::TableFirstInit(_) => {
                    //This is a subscriber event. We skip it at Persist Flow
                }
            }
        }
    } else {
        updates_to_persist_by_table
            .flag_table_to_update(next_events.table_name.as_str(), sync_moment)
            .await;
    }
}
