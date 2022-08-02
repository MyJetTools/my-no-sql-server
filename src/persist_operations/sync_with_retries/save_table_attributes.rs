use std::sync::Arc;

use my_no_sql_core::db::DbTableAttributes;

use crate::app::AppContext;
pub async fn save_table_attributes(
    app: &Arc<AppContext>,
    table_name: &str,
    attrs: &DbTableAttributes,
) {
    let mut attempt_no = 0;
    loop {
        let app_moved = app.clone();
        let table_name_moved = table_name.to_string();
        let attrs_moved = attrs.clone();

        let save_result = tokio::spawn(async move {
            app_moved
                .persist_grpc_service
                .save_table_attrs(table_name_moved.as_str(), attrs_moved)
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
                    "save_table_attributes".to_string(),
                    format!(
                        "Attempt: {attempt_no}. Error saving {table_name}: {:?}",
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
