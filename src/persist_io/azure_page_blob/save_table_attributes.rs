use std::time::Duration;

use my_azure_storage_sdk::AzureStorageConnection;

use crate::{app::logs::Logs, db::DbTableAttributesSnapshot};

pub async fn with_retries(
    logs: &Logs,
    azure_connection: &AzureStorageConnection,
    table_name: &str,
    attr: &DbTableAttributesSnapshot,
) {
    let mut attempt_no = 0;
    loop {
        match super::table::save_attributes(&azure_connection, table_name, &attr).await {
            Ok(_) => {
                logs.add_info(
                    Some(table_name.to_string()),
                    crate::app::logs::SystemProcess::PersistOperation,
                    "save_table_attributes".to_string(),
                    "Saved".to_string(),
                );
                return;
            }
            Err(err) => {
                attempt_no += 1;
                super::blob_errors_handler::handle_azure_blob_error(
                    logs,
                    "save_table_attributes",
                    &err,
                    table_name,
                    azure_connection,
                    attempt_no,
                )
                .await;

                tokio::time::sleep(Duration::from_secs(3)).await;
            }
        }
    }
}
