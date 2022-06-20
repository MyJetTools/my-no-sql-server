use std::sync::Arc;

use crate::app::AppContext;

pub async fn table_list_of_files_loader(app: Arc<AppContext>) {
    while let Some(table_to_load) = app.init_state.get_next_table_to_init_files().await {
        app.persist_io.get_table_files(&table_to_load).await;
    }

    app.init_state.loading_files_for_tables_is_done().await;
}
