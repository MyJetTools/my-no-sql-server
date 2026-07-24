use my_no_sql_sdk::core::rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{app::AppContext, db_sync::EventSource};

use super::{BulkProcessError, BulkProcessScope};

/// Applies the whole accumulated snapshot to the real table.
///
/// Both apply paths clean and re-insert under a single `db_table.data.write()`
/// and emit a single sync event, so readers never observe the intermediate
/// (cleaned but not yet filled) state - which is the entire point of collecting
/// the chunks aside instead of writing them as they arrive.
pub async fn commit(
    app: &AppContext,
    process_id: &str,
    session: Option<&str>,
    event_src: EventSource,
    persist_moment: DateTimeAsMicroseconds,
    now: DateTimeAsMicroseconds,
) -> Result<(), BulkProcessError> {
    let process = app.active_bulk_processes.take(process_id, session).await?;

    // Resolved here as well as at start: the table could have been deleted
    // while the chunks were being uploaded.
    let db_table = crate::db_operations::read::table::get(app, process.table_name.as_str()).await?;

    let scope = process.scope.clone();
    let rows_by_partition = process.into_rows_by_partition();

    match scope {
        BulkProcessScope::Table => {
            crate::db_operations::write::clean_table_and_bulk_insert::execute(
                app,
                db_table,
                rows_by_partition,
                Some(event_src),
                persist_moment,
                now,
            )
            .await?;
        }
        BulkProcessScope::Partition(partition_key) => {
            crate::db_operations::write::clean_partition_and_bulk_insert(
                app,
                &db_table,
                partition_key,
                rows_by_partition,
                event_src,
                persist_moment,
                now,
            )
            .await?;
        }
    }

    Ok(())
}
