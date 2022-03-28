use std::sync::Arc;

use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    app::AppContext,
    db::{DbTable, DbTableData},
    db_sync::{
        states::{DeleteRowsEventSyncData, InitPartitionsSyncData},
        EventSource, SyncEvent,
    },
    utils::LazyVec,
};

pub async fn keep_max_partitions_amount(
    app: &AppContext,
    db_table: Arc<DbTable>,
    max_partitions_amount: usize,
    event_src: EventSource,
    persist_moment: DateTimeAsMicroseconds,
) {
    let partitions_amount = db_table.get_partitions_amount().await;

    if partitions_amount <= max_partitions_amount {
        return;
    }

    let mut table_data = db_table.data.write().await;

    gc_partitions(
        app,
        db_table.as_ref(),
        &mut table_data,
        event_src,
        max_partitions_amount,
        persist_moment,
    );
}

pub async fn keep_max_partitions_amount_and_expire_db_rows(
    app: &AppContext,
    db_table: Arc<DbTable>,
    max_partitions_amount: usize,
    event_src: EventSource,
    persist_moment: DateTimeAsMicroseconds,
    now: DateTimeAsMicroseconds,
) {
    let partitions_amount = db_table.get_partitions_amount().await;

    if partitions_amount <= max_partitions_amount {
        return;
    }

    let mut table_data = db_table.data.write().await;

    gc_partitions(
        app,
        db_table.as_ref(),
        &mut table_data,
        event_src.clone(),
        max_partitions_amount,
        persist_moment,
    );

    expire_rows(
        app,
        &mut table_data,
        now,
        db_table.attributes.get_persist(),
        event_src,
    );
}

fn gc_partitions(
    app: &AppContext,
    db_table: &DbTable,
    table_data: &mut DbTableData,
    event_src: EventSource,
    max_partitions_amount: usize,
    persist_moment: DateTimeAsMicroseconds,
) {
    let mut sync_state =
        InitPartitionsSyncData::new(&table_data, event_src, db_table.attributes.get_persist());

    let gced_partitions_result =
        table_data.gc_and_keep_max_partitions_amount(max_partitions_amount);

    if let Some(gced_partitions) = gced_partitions_result {
        for (partition_key, _) in gced_partitions {
            table_data
                .data_to_persist
                .mark_partition_to_persist(partition_key.as_ref(), persist_moment);

            sync_state.add(partition_key, None);
        }
        app.events_dispatcher
            .dispatch(SyncEvent::InitPartitions(sync_state));
    }
}

fn expire_rows(
    app: &AppContext,
    table_data: &mut DbTableData,
    now: DateTimeAsMicroseconds,
    persist: bool,
    event_src: EventSource,
) {
    let mut all_expired_rows = LazyVec::new();
    for db_partition in table_data.partitions.values_mut() {
        let expired_rows = db_partition.rows.expire_rows(now);

        if let Some(expired_rows) = expired_rows {
            all_expired_rows.extend(expired_rows);
        }
    }

    let sync_data = DeleteRowsEventSyncData::new(table_data, persist, event_src);

    app.events_dispatcher
        .dispatch(SyncEvent::DeleteRows(sync_data));
}
