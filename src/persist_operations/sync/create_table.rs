use crate::{app::AppContext, db::DbTable};

pub async fn create_table(app: &AppContext, db_table: &DbTable) {
    app.persist_io
        .create_table_folder(db_table.name.as_str())
        .await;

    let attrs = db_table.attributes.get_snapshot();

    app.blob_content_cache
        .create_table(db_table.name.as_str(), &attrs)
        .await;

    super::save_table_attributes(app, db_table.name.as_str(), &attrs).await;
}
