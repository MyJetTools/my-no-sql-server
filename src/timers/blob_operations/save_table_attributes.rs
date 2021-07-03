use std::time::Duration;

use my_azure_storage_sdk::AzureConnection;

use crate::{app::AppServices, db::DbTable};

pub async fn with_retries(
    app: &AppServices,
    azure_connection: &AzureConnection,
    db_table: &DbTable,
) {
    let err_delay = Duration::from_secs(3);
    let mut attempt_no = 0;
    loop {
        let attr = db_table.get_attributes().await;

        let result = crate::persistence::blob_repo::save_table_attributes(
            &azure_connection,
            db_table.name.as_str(),
            &attr,
            app,
        )
        .await;

        if result.is_ok() {
            return;
        }

        let err = result.err().unwrap();

        app.logs
            .add_error(
                Some(db_table.name.to_string()),
                "save_partition".to_string(),
                format!(
                    "Can not sync table attributes for the tab;e {}. Doing retry. Attempt: {}",
                    db_table.name, attempt_no
                ),
                Some(format!("{:?}", err)),
            )
            .await;

        attempt_no += 1;

        tokio::time::sleep(err_delay).await;
    }
}
