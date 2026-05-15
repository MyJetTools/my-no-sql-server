use serde_json::Value;

use crate::models::*;

pub struct RequestError {
    pub message: String,
}

impl std::fmt::Display for RequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<reqwest::Error> for RequestError {
    fn from(err: reqwest::Error) -> Self {
        Self {
            message: err.to_string(),
        }
    }
}

fn get_base_url() -> String {
    let settings = dioxus_utils::js::GlobalAppSettings::new();
    let origin = settings.get_origin();
    if origin.ends_with('/') {
        origin[..origin.len() - 1].to_string()
    } else {
        origin.to_string()
    }
}

pub async fn get_status() -> Result<StatusApiModel, RequestError> {
    let url = format!("{}/api/Status", get_base_url());
    let response = reqwest::Client::new().get(&url).send().await?;
    if !response.status().is_success() {
        return Err(RequestError {
            message: format!("Failed to load status: {}", response.status()),
        });
    }
    let result: StatusApiModel = response.json().await?;
    Ok(result)
}

pub async fn get_tables_list() -> Result<Vec<TableListItemApiModel>, RequestError> {
    let url = format!("{}/api/Tables/List", get_base_url());
    let response = reqwest::Client::new().get(&url).send().await?;
    if !response.status().is_success() {
        return Err(RequestError {
            message: format!("Failed to load tables: {}", response.status()),
        });
    }
    let result: Vec<TableListItemApiModel> = response.json().await?;
    Ok(result)
}

pub async fn get_partitions(table_name: &str) -> Result<PartitionsApiModel, RequestError> {
    let url = format!("{}/api/Partitions", get_base_url());
    let response = reqwest::Client::new()
        .get(&url)
        .query(&[("tableName", table_name)])
        .send()
        .await?;
    if !response.status().is_success() {
        return Err(RequestError {
            message: format!("Failed to load partitions: {}", response.status()),
        });
    }
    let result: PartitionsApiModel = response.json().await?;
    Ok(result)
}

pub async fn get_rows(
    table_name: &str,
    partition_key: &str,
) -> Result<Vec<Value>, RequestError> {
    let url = format!("{}/api/Row", get_base_url());
    let response = reqwest::Client::new()
        .get(&url)
        .query(&[("tableName", table_name), ("partitionKey", partition_key)])
        .send()
        .await?;
    if !response.status().is_success() {
        return Err(RequestError {
            message: format!("Failed to load rows: {}", response.status()),
        });
    }
    let result: Vec<Value> = response.json().await?;
    Ok(result)
}

pub async fn delete_row(
    table_name: &str,
    partition_key: &str,
    row_key: &str,
) -> Result<(), RequestError> {
    let url = format!("{}/api/Row", get_base_url());
    let response = reqwest::Client::new()
        .delete(&url)
        .query(&[
            ("tableName", table_name),
            ("partitionKey", partition_key),
            ("rowKey", row_key),
        ])
        .send()
        .await?;
    if !response.status().is_success() {
        return Err(RequestError {
            message: format!("Failed to delete row: {}", response.status()),
        });
    }
    Ok(())
}
