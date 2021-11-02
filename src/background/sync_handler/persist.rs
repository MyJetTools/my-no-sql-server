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
                SyncEvent::UpdateTableAttributes {
                    table: _,
                    attr: _,
                    table_is_just_created: _,
                    persist: _,
                    max_partitions_amount: _,
                } => {
                    app.updates_to_persist_by_table
                        .update_table_attributes(next_events.table_name.as_str(), sync_moment)
                        .await;
                }
                SyncEvent::InitTable(_) => {
                    app.updates_to_persist_by_table
                        .update_table(next_events.table_name.as_str(), sync_moment)
                        .await;
                }
                SyncEvent::InitPartitions(state) => {
                    app.updates_to_persist_by_table
                        .update_partitions(
                            next_events.table_name.as_str(),
                            state.partitions_to_update.keys(),
                            sync_moment,
                        )
                        .await;
                }
                SyncEvent::UpdateRows(state) => {
                    app.updates_to_persist_by_table
                        .update_partitions(
                            next_events.table_name.as_str(),
                            state.updated_rows_by_partition.keys(),
                            sync_moment,
                        )
                        .await;
                }
                SyncEvent::Delete(state) => {
                    if let Some(deleted_partitions) = &state.deleted_partitions {
                        app.updates_to_persist_by_table
                            .update_partitions(
                                next_events.table_name.as_str(),
                                deleted_partitions.keys(),
                                sync_moment,
                            )
                            .await;
                    }

                    if let Some(deleted_rows) = &state.deleted_rows {
                        app.updates_to_persist_by_table
                            .update_partitions(
                                next_events.table_name.as_str(),
                                deleted_rows.keys(),
                                sync_moment,
                            )
                            .await;
                    }
                }
                SyncEvent::DeleteTable { table: _, attr: _ } => {
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
