use std::sync::Arc;

use crate::app::AppContext;

use super::load_tasks::TableToLoad;

pub async fn table_list_of_files_loader(app: Arc<AppContext>, tables: Vec<Arc<TableToLoad>>) {
    for table in tables {
        app.persist_io.get_table_files(&table).await;
    }
}
