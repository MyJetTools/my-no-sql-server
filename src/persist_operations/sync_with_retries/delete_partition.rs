use std::sync::Arc;

use crate::app::AppContext;

pub async fn delete_partition(app: &Arc<AppContext>, table_name: &str, partition_key: &str) {
    let mut attempt_no = 0;
    loop {
        let app_moved = app.clone();
        let table_name_moved = table_name.to_string();
        let partition_key_moved = partition_key.to_string();

        let save_result = tokio::spawn(async move {
            app_moved
                .persist_grpc_service
                .delete_partition(table_name_moved.as_str(), partition_key_moved.as_str())
                .await;
        })
        .await;

        match save_result {
            Ok(_) => {
                return ();
            }
            Err(err) => {
                app.logs.add_error(
                    None,
                    crate::app::logs::SystemProcess::PersistOperation,
                    "delete_partition".to_string(),
                    format!(
                        "Attempt: {attempt_no}. Error deleting partition {table_name}/{partition_key}: {:?}",
                        err
                    ),
                    None,
                );
                attempt_no += 1;
                tokio::time::sleep(app.persist_retry_timeout).await;
            }
        }
    }
}
