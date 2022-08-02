use std::sync::Arc;

use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{app::AppContext, db::DbTableWrapper};

pub async fn save_table(
    app: &Arc<AppContext>,
    db_table_wrapper: &DbTableWrapper,
    persist_moment: DateTimeAsMicroseconds,
) {
    let mut attempt_no = 0;
    loop {
        let app_moved = app.clone();
        let table_name = db_table_wrapper.name.to_string();
        let table_snapshot = db_table_wrapper.get_table_snapshot().await;

        let save_result = tokio::spawn(async move {
            if table_snapshot.by_partition.len() == 0 {
                app_moved
                    .persist_grpc_service
                    .clean_table(table_name, persist_moment)
                    .await;
            } else {
                app_moved
                    .persist_grpc_service
                    .init_table(table_name, table_snapshot)
                    .await;
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
                    "save_table".to_string(),
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
