use std::time::Duration;

use my_azure_storage_sdk::{AzureConnection, AzureStorageError};

use crate::{app::AppServices, db::DbTable, utils::StopWatch};

pub async fn with_retires(
    app: &AppServices,
    azure_connection: &AzureConnection,
    db_table: &DbTable,
) {
    let err_delay = Duration::from_secs(3);
    let mut attempt_no = 0;
    loop {
        let result = sync_table(app, azure_connection, db_table).await;

        if result.is_ok() {
            return;
        }

        let err = result.err().unwrap();

        app.logs
            .add_error(
                Some(db_table.name.to_string()),
                "save_partition".to_string(),
                format!(
                    "Can not sync table {}. Doing retry. Attempt: {}",
                    db_table.name, attempt_no
                ),
                Some(format!("{:?}", err)),
            )
            .await;

        attempt_no += 1;

        tokio::time::sleep(err_delay).await;
    }
}

async fn sync_table(
    app: &AppServices,
    azure_connection: &AzureConnection,
    db_table: &DbTable,
) -> Result<(), AzureStorageError> {
    crate::persistence::blob_repo::clean_table(azure_connection, &db_table.name).await?;
    let partition_keys = db_table.get_partition_keys().await;

    app.logs
        .add_info(
            Some(db_table.name.to_string()),
            "sync_table".to_string(),
            format!("Synching {} partitions", partition_keys.len()),
        )
        .await;

    let mut sb = StopWatch::new();
    sb.start();

    for partition_key in &partition_keys {
        super::save_partition::with_retries(app, azure_connection, db_table, partition_key).await;
    }

    sb.pause();
    app.logs
        .add_info(
            Some(db_table.name.to_string()),
            "sync_table".to_string(),
            format!(
                "Synced {} partitions in {:?}",
                partition_keys.len(),
                sb.duration()
            ),
        )
        .await;

    Ok(())
}
