use std::sync::Arc;

use my_no_sql_sdk::core::db::{DbRow, DbTableName, PartitionKey};
use my_no_sql_sdk::server::DbTableWrapper;

use crate::{app::AppContext, persist_markers::SyncRowJobDescription};

pub async fn save_rows(
    app: &AppContext,
    db_table_name: &DbTableName,
    jobs: Vec<SyncRowJobDescription>,
) {
    let db_table = match app.db.get_table(db_table_name.as_str()).await {
        Some(db_table) => db_table,
        None => {
            super::scripts::delete_table(app, &db_table_name).await;
            return;
        }
    };

    let found_rows = find_and_sort_rows(&db_table, jobs).await;

    for partition_to_delete in found_rows.partitions_to_delete {
        super::scripts::delete_partition(app, db_table_name, &partition_to_delete).await;
    }

    if found_rows.rows_to_delete.len() > 0 {
        super::scripts::delete_rows(app, db_table_name, &found_rows.rows_to_delete).await;
    }

    if found_rows.rows_to_save.len() > 0 {
        super::scripts::save_rows(app, db_table_name, &found_rows.rows_to_save).await;
    }
}

async fn find_and_sort_rows(
    db_table: &DbTableWrapper,
    jobs: Vec<SyncRowJobDescription>,
) -> FoundRowsToSync {
    let mut result = FoundRowsToSync::default();
    let db_table_access = db_table.data.read().await;
    for job in jobs {
        match db_table_access.partitions.get(job.partition_key.as_str()) {
            Some(db_partition) => {
                for job_db_row in job.items {
                    match db_partition.get_row(job_db_row.get_row_key()) {
                        Some(db_row) => {
                            result.rows_to_save.push(db_row.clone());
                        }
                        None => {
                            result.rows_to_delete.push(job_db_row);
                        }
                    }
                }
            }
            None => {
                result.partitions_to_delete.push(job.partition_key.clone());
            }
        }
    }
    result
}

#[derive(Default)]
pub struct FoundRowsToSync {
    rows_to_save: Vec<Arc<DbRow>>,
    rows_to_delete: Vec<Arc<DbRow>>,
    partitions_to_delete: Vec<PartitionKey>,
}
