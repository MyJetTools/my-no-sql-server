use std::sync::Arc;

use crate::app::AppContext;

pub async fn table_list_of_files_loader(app: Arc<AppContext>, table_names: Vec<String>) {
    for table_name in table_names {
        app.persist_io
            .get_table_files(&table_name, &app.init_state)
            .await;
    }
}
