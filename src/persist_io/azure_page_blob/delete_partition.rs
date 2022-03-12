use std::time::Duration;

use my_azure_storage_sdk::AzureStorageConnection;

use crate::app::logs::{Logs, SystemProcess};

pub async fn with_retries(
    logs: &Logs,
    azure_connection: &AzureStorageConnection,
    table_name: &str,
    partition_key: &str,
) {
    let mut attempt_no = 0;
    loop {
        let result = super::partition::delete(azure_connection, table_name, partition_key).await;

        if result.is_ok() {
            logs.add_info(
                Some(table_name.to_string()),
                crate::app::logs::SystemProcess::PersistOperation,
                "delete_partition".to_string(),
                "Saved".to_string(),
            );
            return;
        }

        let err = result.err().unwrap();

        attempt_no += 1;

        logs.add_error(
            Some(table_name.to_string()),
            SystemProcess::PersistOperation,
            "delete_partition".to_string(),
            format!("PartitionKey: {}, Attempt: {}", partition_key, attempt_no),
            Some(format!("{:?}", err)),
        );

        tokio::time::sleep(Duration::from_secs(3)).await;
    }
}
