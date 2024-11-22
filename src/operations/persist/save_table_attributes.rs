use my_no_sql_sdk::core::db::DbTableName;

use crate::app::AppContext;

pub async fn save_table_attributes(app: &AppContext, table_name: &DbTableName) {
    match app.db.get_table(table_name.as_str()).await {
        Some(db_table) => {
            let attr = db_table.get_attributes().await;
            app.repo.save_table_metadata(&table_name, &attr).await;
        }
        None => {
            super::scripts::delete_table(app, &table_name).await;
        }
    }
}
