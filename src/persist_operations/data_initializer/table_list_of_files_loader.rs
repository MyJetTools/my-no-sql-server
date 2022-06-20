use std::sync::Arc;

use crate::app::AppContext;

pub async fn table_list_of_files_loader(app: Arc<AppContext>) {
    while let Some(table_name) = app.init_state.get_next_table_to_load_list_of_files().await {
        app.persist_io
            .get_table_files(&table_name, &app.init_state)
            .await;
    }
}
