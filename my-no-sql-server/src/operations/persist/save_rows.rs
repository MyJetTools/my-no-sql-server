use my_no_sql_sdk::core::db::DbTableName;

use crate::{app::AppContext, persist_markers::SyncRowJobDescription};

/// Handles a `SyncRows` task. With per-partition storage there is no row-level
/// write: every partition touched by the dirty rows is re-serialized whole
/// (deleted rows simply fall out of the fresh snapshot), or its blob is removed
/// if the partition no longer exists.
pub async fn save_rows(
    app: &AppContext,
    db_table_name: &DbTableName,
    jobs: Vec<SyncRowJobDescription>,
) {
    let db_table = match app.db.get_table(db_table_name.as_str()) {
        Some(db_table) => db_table,
        None => {
            super::scripts::delete_table(app, db_table_name).await;
            return;
        }
    };

    for job in jobs {
        match db_table.get_partition_snapshot(job.partition_key.as_str()) {
            Some(snapshot) => {
                super::scripts::sync_partition_snapshot(
                    app,
                    db_table_name,
                    &job.partition_key,
                    snapshot,
                )
                .await;
            }
            None => {
                super::scripts::delete_partition(app, db_table_name, &job.partition_key).await;
            }
        }
    }
}
