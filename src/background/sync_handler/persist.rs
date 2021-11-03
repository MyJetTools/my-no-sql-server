use std::time::Duration;

use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    app::{AppContext, NextEventsToHandle},
    db_sync::SyncEvent,
};

const FIVE_SECS: i64 = Duration::from_secs(5).as_micros() as i64;

pub async fn execute(app: &AppContext, next_events: &NextEventsToHandle) {
    let mut sync_moment = DateTimeAsMicroseconds::now();

    sync_moment.unix_microseconds += FIVE_SECS; //TODO - Hardcoded 5 seconds persistence time

    let get_table_result = app.db.get_table(next_events.table_name.as_str()).await;

    if get_table_result.is_some() {
        for next_event in &next_events.events {
            match &next_event {
                SyncEvent::UpdateTableAttributes(_) => {
                    app.updates_to_persist_by_table
                        .update_table_attributes(next_events.table_name.as_str(), sync_moment)
                        .await;
                }
                SyncEvent::InitTable(data) => {
                    if data.table_data.persist {
                        app.updates_to_persist_by_table
                            .update_table(next_events.table_name.as_str(), sync_moment)
                            .await;
                    }
                }
                SyncEvent::InitPartitions(data) => {
                    if data.table_data.persist {
                        app.updates_to_persist_by_table
                            .update_partitions(
                                next_events.table_name.as_str(),
                                data.partitions_to_update.keys(),
                                sync_moment,
                            )
                            .await;
                    }
                }
                SyncEvent::UpdateRows(data) => {
                    if data.table_data.persist {
                        app.updates_to_persist_by_table
                            .update_partitions(
                                next_events.table_name.as_str(),
                                data.updated_rows_by_partition.keys(),
                                sync_moment,
                            )
                            .await;
                    }
                }
                SyncEvent::Delete(data) => {
                    if data.table_data.persist {
                        if let Some(deleted_partitions) = &data.deleted_partitions {
                            app.updates_to_persist_by_table
                                .update_partitions(
                                    next_events.table_name.as_str(),
                                    deleted_partitions.keys(),
                                    sync_moment,
                                )
                                .await;
                        }
                        if let Some(deleted_rows) = &data.deleted_rows {
                            app.updates_to_persist_by_table
                                .update_partitions(
                                    next_events.table_name.as_str(),
                                    deleted_rows.keys(),
                                    sync_moment,
                                )
                                .await;
                        }
                    }
                }
                SyncEvent::DeleteTable(_) => {
                    app.updates_to_persist_by_table
                        .update_table(next_events.table_name.as_str(), sync_moment)
                        .await;
                }
            }
        }
    } else {
        app.updates_to_persist_by_table
            .update_table(next_events.table_name.as_str(), sync_moment)
            .await;
    }
}
