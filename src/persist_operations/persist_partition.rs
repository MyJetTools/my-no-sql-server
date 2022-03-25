use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    app::AppContext,
    db::{db_snapshots::DbPartitionSnapshot, DbTable},
    persist_operations::blob_content_cache::BlobPartitionUpdateTimeResult,
};

pub async fn execute(app: &AppContext, db_table: &DbTable, partition_key: &str) {
 
}
