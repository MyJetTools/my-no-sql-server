use std::sync::Arc;

use crate::{app::AppContext, db::DbTableWrapper};

pub async fn save_partition(
    app: &Arc<AppContext>,
    db_table_wrapper: &DbTableWrapper,
    partition_key: &str,
) {
    let mut attempt_no = 0;
    loop {
        let app_moved = app.clone();
        let table_name = db_table_wrapper.name.to_string();
        let partition_key_moved = partition_key.to_string();
        let partition_snapshot = db_table_wrapper.get_partition_snapshot(partition_key).await;

        let save_result = tokio::spawn(async move {
            match partition_snapshot {
                Some(partition_snapshot) => {
                    app_moved
                        .persist_grpc_service
                        .init_partition(table_name.as_str(), partition_snapshot)
                        .await;
                }
                None => {
                    app_moved
                        .persist_grpc_service
                        .delete_partition(table_name.as_str(), partition_key_moved.as_str())
                        .await;
                }
            }
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
                    "save_partition".to_string(),
                    format!(
                        "Attempt: {attempt_no}. Error saving {}: {:?}",
                        db_table_wrapper.name, err
                    ),
                    None,
                );
                attempt_no += 1;
                tokio::time::sleep(app.persist_retry_timeout).await;
            }
        }
    }
}
