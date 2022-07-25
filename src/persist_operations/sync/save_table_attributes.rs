use my_no_sql_core::db::DbTableAttributesSnapshot;

use super::super::serializers;
use crate::{app::AppContext, persist_io::TableFile};
pub async fn save_table_attributes(
    app: &AppContext,
    table_name: &str,
    attrs: &DbTableAttributesSnapshot,
) {
    app.persist_io.create_table_folder(table_name).await;

    let content = serializers::table_attrs::serialize(attrs);

    app.persist_io
        .save_table_file(table_name, &TableFile::TableAttributes, content)
        .await;

    app.blob_content_cache
        .update_table_attributes(table_name, attrs.clone())
        .await;
}
