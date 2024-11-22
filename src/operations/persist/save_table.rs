use my_no_sql_sdk::core::db::DbTableName;

use crate::app::AppContext;

pub async fn save_table(app: &AppContext, table_name: &DbTableName) {
    match app.db.get_table(table_name.as_str()).await {
        Some(db_table) => {
            let table_snapshot = db_table.get_table_snapshot().await;
            super::scripts::sync_table_snapshot(app, table_name, table_snapshot).await;
        }
        None => {
            super::scripts::delete_table(app, &table_name).await;
        }
    }
}
