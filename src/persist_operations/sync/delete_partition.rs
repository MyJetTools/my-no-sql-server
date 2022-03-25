use crate::{
    app::AppContext, persist_io::TableFile,
    persist_operations::blob_content_cache::BlobPartitionUpdateTimeResult,
};

pub async fn delete_partition(app: &AppContext, table_name: &str, partition_key: &str) {
    let partition_in_blob = app.blob_content_cache.get(table_name, partition_key).await;

    match partition_in_blob {
        BlobPartitionUpdateTimeResult::Ok(_) => {
            app.persist_io
                .delete_table_file(
                    table_name,
                    &TableFile::DbPartition(partition_key.to_string()),
                )
                .await;

            app.blob_content_cache
                .delete_table_partition(table_name, partition_key)
                .await;
        }
        BlobPartitionUpdateTimeResult::TableNotFound => {}
        BlobPartitionUpdateTimeResult::PartitionNoFound => {}
    }
}
