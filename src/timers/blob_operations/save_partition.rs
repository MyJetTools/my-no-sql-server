use std::time::Duration;

use my_azure_storage_sdk::{AzureConnection, AzureStorageError};

use crate::{app::AppServices, db::DbTable};

pub async fn with_retries(
    app: &AppServices,
    azure_connection: &AzureConnection,
    db_table: &DbTable,
    partition_key: &str,
) {
    let err_delay = Duration::from_secs(3);
    let mut attempt_no = 0;
    loop {
        let result = save_partition(azure_connection, db_table, partition_key).await;

        if result.is_ok() {
            return;
        }

        let err = result.err().unwrap();

        app.logs
            .add_error(
                Some(db_table.name.to_string()),
                "save_partition".to_string(),
                format!(
                    "Can not sync partition {}. Doing retry. Attempt: {}",
                    partition_key, attempt_no
                ),
                Some(format!("{:?}", err)),
            )
            .await;

        attempt_no += 1;

        tokio::time::sleep(err_delay).await;
    }
}

async fn save_partition(
    azure_connection: &AzureConnection,
    db_table: &DbTable,
    partition_key: &str,
) -> Result<(), AzureStorageError> {
    let partition_snapshot = db_table.get_partition_snapshot_as_json(partition_key).await;

    match partition_snapshot {
        Some(json) => {
            crate::persistence::blob_repo::save_partition(
                azure_connection,
                db_table.name.as_str(),
                partition_key,
                json,
            )
            .await
        }
        None => {
            crate::persistence::blob_repo::delete_partition(
                azure_connection,
                db_table.name.as_str(),
                partition_key,
            )
            .await
        }
    }
}
