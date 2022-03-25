use std::sync::Arc;

use crate::app::AppContext;

pub async fn delete_table(app: Arc<AppContext>, table_name: String) {
    if !app.blob_content_cache.has_table(table_name.as_str()).await {
        return;
    }

    app.persist_io
        .delete_table_folder(table_name.as_str())
        .await;

    app.blob_content_cache
        .delete_table(table_name.as_str())
        .await;
}
