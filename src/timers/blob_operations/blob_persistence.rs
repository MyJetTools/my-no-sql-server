use std::{sync::Arc, time::Duration};

use my_azure_storage_sdk::AzureConnection;

use crate::app::{logs::SystemProcess, AppServices};

use super::BlobOperationsToBeMade;

pub async fn start(app: Arc<AppServices>, azure_connection: Arc<AzureConnection>) {
    app.logs
        .add_info(
            None,
            SystemProcess::BlobOperation,
            "Timer blob persistence initialization".to_string(),
            "Started".to_string(),
        )
        .await;
    let delay = Duration::from_secs(5);

    loop {
        tokio::time::sleep(delay).await;
        iterate(app.as_ref(), azure_connection.as_ref()).await;
    }
}

async fn iterate(app: &AppServices, azure_connection: &AzureConnection) {
    let events = app.queue_to_persist.dequeue().await;

    if events.is_none() {
        return;
    }

    let events = events.unwrap();

    let operations_to_be_made = BlobOperationsToBeMade::new(&events.1);

    let db_table = app.db.get_table(&events.0).await;

    if let Err(err) = &db_table {
        app.logs
            .add_error(
                Some(events.0.to_string()),
                SystemProcess::BlobOperation,
                "blob_persistece_handler.execute()".to_string(),
                "Some how blob is not found. BUG".to_string(),
                Some(format!("{:?}", err)),
            )
            .await;
    }

    let db_table = db_table.unwrap();

    if operations_to_be_made.sync_attributes {
        super::save_table_attributes::with_retries(app, azure_connection, db_table.as_ref()).await;
    }

    if operations_to_be_made.sync_table {
        super::save_table::with_retires(app, azure_connection, db_table.as_ref()).await;
    } else {
        for partition_key in operations_to_be_made.sync_partitions.keys() {
            super::save_partition::with_retries(
                app,
                azure_connection,
                db_table.as_ref(),
                &partition_key,
            )
            .await;
        }
    }
}
