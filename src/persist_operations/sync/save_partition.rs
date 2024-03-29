use my_no_sql_sdk::core::db::PartitionKey;
use my_no_sql_server_core::DbTableWrapper;

use crate::{
    app::AppContext, persist_operations::blob_content_cache::BlobPartitionUpdateTimeResult,
};

use super::super::sync;

pub async fn save_partition(
    app: &AppContext,
    db_table: &DbTableWrapper,
    partition_key: PartitionKey,
) {
    let get_blob_content_cache = app
        .blob_content_cache
        .get(db_table.name.as_str(), partition_key.as_str())
        .await;

    let partition_snapshot = db_table
        .get_partition_snapshot(partition_key.as_str())
        .await;

    match get_blob_content_cache {
        BlobPartitionUpdateTimeResult::Ok(blob_date_time) => {
            if partition_snapshot.is_none() {
                sync::delete_partition(app, db_table.name.as_str(), partition_key).await;
                return;
            }

            let partition_snapshot = partition_snapshot.unwrap();

            if partition_snapshot.last_write_moment.unix_microseconds
                > blob_date_time.unix_microseconds
            {
                sync::upload_partition(app, db_table.name.as_str(), partition_snapshot).await;
            }
        }
        BlobPartitionUpdateTimeResult::TableNotFound => {
            if let Some(snapshot) = partition_snapshot {
                sync::create_table(app, db_table).await;
                sync::upload_partition(app, db_table.name.as_str(), snapshot).await;
            }
        }
        BlobPartitionUpdateTimeResult::PartitionNoFound => {
            if let Some(snapshot) = partition_snapshot {
                sync::upload_partition(app, db_table.name.as_str(), snapshot).await;
            }
        }
    }
}
