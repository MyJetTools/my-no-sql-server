use std::sync::Arc;

use my_azure_storage_sdk::AzureConnectionWithTelemetry;
use my_azure_storage_sdk::{blob::BlobApi, AzureStorageError};

use my_azure_storage_sdk::blob_container::BlobContainersApi;
use rust_extensions::date_time::DateTimeAsMicroseconds;
use tokio::task::JoinHandle;

use crate::db::DbTableAttributesSnapshot;
use crate::{
    db::{DbPartition, DbTableData},
    db_json_entity::{DbJsonEntity, JsonTimeStamp},
    json::array_parser::ArrayToJsonObjectsSplitter,
};
use my_app_insights::AppInsightsTelemetry;

use super::metadata::{TableMetadataFileContract, METADATA_BLOB_NAME};

pub async fn load(
    azure_connection: Arc<AzureConnectionWithTelemetry<AppInsightsTelemetry>>,
    table_name: &str,
    init_threads_amount: usize,
) -> Result<(DbTableData, DbTableAttributesSnapshot), AzureStorageError> {
    let blobs = azure_connection.get_list_of_blobs(&table_name).await?;

    let now = DateTimeAsMicroseconds::now();

    let mut db_table_data = DbTableData::new(table_name.to_string(), now);

    let mut tasks = Vec::new();

    let mut db_table_attirbutes = DbTableAttributesSnapshot {
        created: DateTimeAsMicroseconds::now(),
        persist: true,
        max_partitions_amount: None,
    };

    for blob_name in blobs {
        let handle = tokio::spawn(load_blob(
            azure_connection.clone(),
            table_name.to_string(),
            blob_name,
        ));

        tasks.push(handle);

        if tasks.len() == init_threads_amount {
            init_to_db_table(&mut db_table_data, &mut db_table_attirbutes, &mut tasks).await;
        }
    }

    init_to_db_table(&mut db_table_data, &mut db_table_attirbutes, &mut tasks).await;

    return Ok((db_table_data, db_table_attirbutes));
}

async fn init_to_db_table(
    db_table_data: &mut DbTableData,
    table_attributes: &mut DbTableAttributesSnapshot,
    tasks: &mut Vec<JoinHandle<LoadBlobResult>>,
) {
    for task in tasks.drain(..) {
        let result = task.await.unwrap();

        match result {
            LoadBlobResult::Metadata(meta_data) => {
                table_attributes.max_partitions_amount = meta_data.max_partitions_amount;
                table_attributes.persist = meta_data.persist;
            }
            LoadBlobResult::DbPartition {
                partition_key,
                db_partition,
            } => {
                db_table_data.init_partition(partition_key, db_partition);
            }
        }
    }
}

pub enum LoadBlobResult {
    Metadata(TableMetadataFileContract),
    DbPartition {
        partition_key: String,
        db_partition: DbPartition,
    },
}

async fn load_blob(
    azure_connection: Arc<AzureConnectionWithTelemetry<AppInsightsTelemetry>>,
    table_name: String,
    blob_name: String,
) -> LoadBlobResult {
    let raw = azure_connection
        .download_blob(&table_name, blob_name.as_str())
        .await
        .unwrap();

    if blob_name == METADATA_BLOB_NAME {
        let table_metadata = TableMetadataFileContract::parse(raw.as_slice());

        return LoadBlobResult::Metadata(table_metadata);
    }

    let partition_name = base64::decode(blob_name.as_str()).unwrap();
    let partition_key = String::from_utf8(partition_name).unwrap();

    let mut db_partition = DbPartition::new();

    for db_entity_json in raw.as_slice().split_array_json_to_objects() {
        let db_entity_json = db_entity_json.unwrap();
        let db_entity = DbJsonEntity::parse(db_entity_json);

        if let Err(err) = db_entity {
            println!("{}", std::str::from_utf8(db_entity_json).unwrap());
            panic!("{:?}", err);
        }

        let db_entity = db_entity.unwrap();

        let db_row = if let Some(time_stamp) = db_entity.time_stamp {
            let time_stamp = JsonTimeStamp::parse_or_now(time_stamp);
            db_entity.restore_db_row(&time_stamp)
        } else {
            let time_stamp = JsonTimeStamp::now();
            db_entity.to_db_row(&time_stamp)
        };

        db_partition.insert(Arc::new(db_row));
    }

    return LoadBlobResult::DbPartition {
        partition_key,
        db_partition,
    };
}
