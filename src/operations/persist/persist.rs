use std::sync::Arc;

use my_no_sql_sdk::core::rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    app::{AppContext, DbNamespace},
    persist_markers::PersistTask,
};

pub async fn persist(app: &Arc<AppContext>) -> bool {
    // Single-flight: the timer, the Force-Persist HTTP action and the shutdown
    // drain may call this concurrently; overlapping tasks would break the
    // in-flight-write-then-cleanup ordering that table deletion relies on.
    //
    // One lock for the whole server rather than one per namespace: that ordering
    // is what makes a table delete safe, and walking the namespaces is cheap.
    let _single_flight = app.persist_call_lock.lock().await;

    let start_time = DateTimeAsMicroseconds::now();

    let now = if app.states.is_shutting_down() {
        None
    } else {
        Some(start_time)
    };

    // Each namespace keeps a persist queue of its own. Take the first task we
    // find and let the caller come back for more — one task per call keeps every
    // namespace moving instead of draining one of them under a time budget.
    for db_namespace in app.namespaces.get_all() {
        let persist_task = match db_namespace.persist_markers.get_persist_task(now).await {
            Some(persist_task) => persist_task,
            None => continue,
        };

        execute_persist_task(&db_namespace, persist_task, start_time).await;

        return true;
    }

    false
}

async fn execute_persist_task(
    db_namespace: &Arc<DbNamespace>,
    persist_task: PersistTask,
    start_time: DateTimeAsMicroseconds,
) {
    let db_table_name = match persist_task {
        PersistTask::SaveTableAttributes(db_table_name) => {
            super::save_table_attributes(db_namespace, &db_table_name).await;
            db_table_name
        }
        PersistTask::SyncTable(db_table_name) => {
            super::save_table(db_namespace, &db_table_name).await;
            db_table_name
        }
        PersistTask::SyncPartition {
            table_name,
            partition_key,
        } => {
            super::save_partition(db_namespace, &table_name, partition_key).await;
            table_name
        }
        PersistTask::SyncRows { table_name, jobs } => {
            super::save_rows(db_namespace, &table_name, jobs).await;
            table_name
        }
    };

    let now = DateTimeAsMicroseconds::now();
    let duration = now.duration_since(start_time).as_positive_or_zero();

    db_namespace
        .persist_markers
        .set_last_persist_time(&db_table_name, now, duration)
        .await;
}
