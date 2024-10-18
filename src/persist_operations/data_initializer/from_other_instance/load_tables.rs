use flurl::FlUrl;
use serde::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct TableMyNoSqlServerContract {
    pub name: String,
    pub persist: Option<bool>,
    #[serde(rename = "maxPartitionsAmount")]
    pub max_partitions_amount: Option<i64>,
    pub max_rows_per_partition_amount: Option<i64>,
}

pub async fn load_tables(url: &str) -> Vec<TableMyNoSqlServerContract> {
    let mut response = FlUrl::new(url)
        .append_path_segment("api")
        .append_path_segment("Tables")
        .append_path_segment("List")
        .get()
        .await
        .unwrap();

    let body = response.get_body_as_slice().await.unwrap();

    serde_json::from_slice(body).unwrap()
}
