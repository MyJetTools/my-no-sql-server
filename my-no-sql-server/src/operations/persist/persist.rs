use std::sync::Arc;

use my_no_sql_sdk::core::rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{app::AppContext, persist_markers::PersistTask};

pub async fn persist(app: &Arc<AppContext>) -> bool {
    // Single-flight: the timer, the Force-Persist HTTP action and the shutdown
    // drain may call this concurrently; overlapping tasks would break the
    // in-flight-write-then-cleanup ordering that table deletion relies on.
    let _single_flight = app.persist_call_lock.lock().await;

    let start_time = DateTimeAsMicroseconds::now();

    let now = if app.states.is_shutting_down() {
        None
    } else {
        Some(start_time)
    };

    let persist_task = if let Some(persist_task) = app.persist_markers.get_persist_task(now).await {
        persist_task
    } else {
        return false;
    };

    let db_table_name = match persist_task {
        PersistTask::SaveTableAttributes(db_table_name) => {
            super::save_table_attributes(app, &db_table_name).await;
            db_table_name
        }
        PersistTask::SyncTable(db_table_name) => {
            super::save_table(app, &db_table_name).await;
            db_table_name
        }
        PersistTask::SyncPartition {
            table_name,
            partition_key,
        } => {
            super::save_partition(app, &table_name, partition_key).await;
            table_name
        }
        PersistTask::SyncRows { table_name, jobs } => {
            super::save_rows(app, &table_name, jobs).await;
            table_name
        }
    };

    let now = DateTimeAsMicroseconds::now();
    let duration = now.duration_since(start_time).as_positive_or_zero();

    app.persist_markers
        .set_last_persist_time(&db_table_name, now, duration)
        .await;

    true
}
