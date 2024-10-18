use std::sync::Arc;

use flurl::FlUrl;
use my_no_sql_sdk::core::{db::DbRow, db_json_entity::DbJsonEntity};

pub async fn load_rows(url: &str, table_name: &str) -> Vec<Arc<DbRow>> {
    let mut response = FlUrl::new(url)
        .append_path_segment("api")
        .append_path_segment("Row")
        .append_query_param("tableName", Some(table_name))
        .get()
        .await
        .unwrap();

    let body = response.get_body_as_slice().await.unwrap();

    DbJsonEntity::restore_as_vec(body).unwrap()
}
