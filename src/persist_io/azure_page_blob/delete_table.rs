use std::{sync::Arc, time::Duration};

use my_azure_storage_sdk::AzureStorageConnection;

use crate::app::logs::Logs;

pub async fn with_retries(
    logs: Arc<Logs>,
    azure_connection: Arc<AzureStorageConnection>,
    table_name: &str,
) {
    let mut attempt_no = 0;
    loop {
        let result = super::table::delete(azure_connection.as_ref(), table_name).await;

        if result.is_ok() {
            return;
        }

        let err = result.err().unwrap();

        attempt_no += 1;

        logs.add_error(
            Some(table_name.to_string()),
            crate::app::logs::SystemProcess::PersistOperation,
            "delete_table".to_string(),
            format!("Attempt: {}", attempt_no),
            Some(format!("{:?}", err)),
        )
        .await;

        tokio::time::sleep(Duration::from_secs(3)).await;
    }
}
