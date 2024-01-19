use my_no_sql_sdk::core::db::PartitionKey;

use crate::{
    app::AppContext, persist_io::TableFile,
    persist_operations::blob_content_cache::BlobPartitionUpdateTimeResult,
};

pub async fn delete_partition(app: &AppContext, table_name: &str, partition_key: PartitionKey) {
    let partition_in_blob = app
        .blob_content_cache
        .get(table_name, partition_key.as_str())
        .await;

    match partition_in_blob {
        BlobPartitionUpdateTimeResult::Ok(_) => {
            app.persist_io
                .delete_table_file(table_name, &TableFile::DbPartition(partition_key.clone()))
                .await;

            app.blob_content_cache
                .delete_table_partition(table_name, partition_key.as_str())
                .await;
        }
        BlobPartitionUpdateTimeResult::TableNotFound => {}
        BlobPartitionUpdateTimeResult::PartitionNoFound => {}
    }
}
