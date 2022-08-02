use std::sync::Arc;

use crate::{app::AppContext, db::DbTableWrapper};

pub async fn delete_table(app: Arc<AppContext>, db_table_wrapper: &DbTableWrapper) {
    {
        let read_access = db_table_wrapper.data.read().await;
        if !read_access
            .persisted_table_data
            .has_table(db_table_wrapper.name.as_str())
        {
            return;
        }
    }

    app.persist_io
        .delete_table_folder(db_table_wrapper.name.as_str())
        .await;

    let mut write_access = db_table_wrapper.data.write().await;
    write_access
        .persisted_table_data
        .delete_table(db_table_wrapper.name.as_str())
        .await;
}
