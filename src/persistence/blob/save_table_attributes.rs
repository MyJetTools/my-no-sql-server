use std::time::Duration;

use my_azure_storage_sdk::AzureConnection;

use crate::{
    app::{logs::SystemProcess, AppServices},
    db::DbTableAttributes,
};

pub async fn with_retries(
    app: &AppServices,
    azure_connection: &AzureConnection,
    table_name: &str,
    attr: &DbTableAttributes,
) {
    let mut attempt_no = 0;
    loop {
        let result = super::repo::save_table_attributes(&azure_connection, table_name, &attr).await;

        if result.is_ok() {
            app.logs
                .add_info(
                    Some(table_name.to_string()),
                    crate::app::logs::SystemProcess::BlobOperation,
                    "save_table_attributes".to_string(),
                    "Saved".to_string(),
                )
                .await;
            return;
        }

        let err = result.err().unwrap();

        attempt_no += 1;

        app.logs
            .add_error(
                Some(table_name.to_string()),
                SystemProcess::BlobOperation,
                "save_table_attributes".to_string(),
                format!("Can not sync table attributes.Attempt: {}", attempt_no),
                Some(format!("{:?}", err)),
            )
            .await;

        tokio::time::sleep(Duration::from_secs(3)).await;
    }
}
