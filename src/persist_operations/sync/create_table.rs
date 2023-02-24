use my_no_sql_server_core::DbTableWrapper;

use crate::app::AppContext;

pub async fn create_table(app: &AppContext, db_table: &DbTableWrapper) {
    app.persist_io
        .create_table_folder(db_table.name.as_str())
        .await;

    let attrs = db_table.get_attributes().await;

    app.blob_content_cache
        .create_table(db_table.name.as_str(), &attrs)
        .await;

    super::save_table_attributes(app, db_table.name.as_str(), &attrs).await;
}
