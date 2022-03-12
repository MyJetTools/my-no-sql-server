use crate::{app::AppContext, db::DbTable};

pub async fn execute(app: &AppContext, db_table: &DbTable) {
    let attr = db_table.attributes.get_snapshot();
    app.persist_io
        .save_table_attributes(db_table.name.as_str(), &attr)
        .await;
}
