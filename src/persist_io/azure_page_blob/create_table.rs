use std::time::Duration;

use my_azure_storage_sdk::{AzureStorageConnection, AzureStorageError};

use crate::{app::logs::Logs, db::DbTableAttributesSnapshot};

pub async fn with_retries(
    logs: &Logs,
    azure_connection: &AzureStorageConnection,
    table_name: &str,
    attr: &DbTableAttributesSnapshot,
) {
    let mut attempt_no = 0;
    loop {
        let result = create_table(azure_connection, table_name, attr).await;

        if result.is_ok() {
            return;
        }

        let err = result.err().unwrap();

        attempt_no += 1;

        logs.add_error(
            Some(table_name.to_string()),
            crate::app::logs::SystemProcess::PersistOperation,
            "create_table".to_string(),
            format!("Attempt: {}", attempt_no),
            Some(format!("{:?}", err)),
        );

        tokio::time::sleep(Duration::from_secs(3)).await;
    }
}

async fn create_table(
    azure_connection: &AzureStorageConnection,
    table_name: &str,
    attr: &DbTableAttributesSnapshot,
) -> Result<(), AzureStorageError> {
    super::table::create_if_not_exists(azure_connection, table_name).await?;
    super::table::save_attributes(azure_connection, table_name, attr).await?;

    Ok(())
}
