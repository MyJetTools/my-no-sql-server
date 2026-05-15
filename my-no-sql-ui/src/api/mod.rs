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

pub async fn get_ui_settings() -> Result<crate::settings::UiServerSettings, RequestError> {
    let url = format!("{}/api/Settings", get_base_url());
    let response = reqwest::Client::new().get(&url).send().await?;
    if !response.status().is_success() {
        // Treat any non-success as "settings not available yet" — fall
        // back to defaults so the UI keeps working on an older server.
        return Ok(crate::settings::UiServerSettings::default());
    }
    #[derive(serde::Deserialize)]
    struct Payload {
        #[serde(rename = "warnMs")]
        warn_ms: u32,
        #[serde(rename = "badMs")]
        bad_ms: u32,
        #[serde(rename = "mcpWritePasswordSet", default)]
        mcp_write_password_set: bool,
    }
    let p: Payload = response.json().await?;
    Ok(crate::settings::UiServerSettings {
        thresholds: crate::settings::HealthThresholds {
            warn_ms: p.warn_ms,
            bad_ms: p.bad_ms,
        },
        mcp_write_password_set: p.mcp_write_password_set,
    })
}

pub async fn get_health_thresholds() -> Result<crate::settings::HealthThresholds, RequestError> {
    Ok(get_ui_settings().await?.thresholds)
}

pub async fn set_health_thresholds(
    t: crate::settings::HealthThresholds,
) -> Result<(), RequestError> {
    let url = format!("{}/api/Settings", get_base_url());
    #[derive(serde::Serialize)]
    struct Payload {
        #[serde(rename = "warnMs")]
        warn_ms: u32,
        #[serde(rename = "badMs")]
        bad_ms: u32,
    }
    let response = reqwest::Client::new()
        .post(&url)
        .json(&Payload {
            warn_ms: t.warn_ms,
            bad_ms: t.bad_ms,
        })
        .send()
        .await?;
    if !response.status().is_success() {
        return Err(RequestError {
            message: format!("Failed to save settings: {}", response.status()),
        });
    }
    Ok(())
}

/// Sets or clears the MCP write password via the dedicated endpoint
/// POST `/api/Settings/McpWritePassword`. Pass `""` to clear. Value is
/// hashed (salt + SHA-256) on the server before persistence.
pub async fn set_mcp_write_password(value: &str) -> Result<(), RequestError> {
    let url = format!("{}/api/Settings/McpWritePassword", get_base_url());
    #[derive(serde::Serialize)]
    struct Payload<'a> {
        password: &'a str,
    }
    let response = reqwest::Client::new()
        .post(&url)
        .json(&Payload { password: value })
        .send()
        .await?;
    if !response.status().is_success() {
        return Err(RequestError {
            message: format!("Failed to save MCP write password: {}", response.status()),
        });
    }
    Ok(())
}

pub async fn get_snapshots_list() -> Result<Vec<String>, RequestError> {
    let url = format!("{}/api/Backup/List", get_base_url());
    let response = reqwest::Client::new().get(&url).send().await?;
    if !response.status().is_success() {
        return Err(RequestError {
            message: format!("Failed to load snapshots: {}", response.status()),
        });
    }
    let result: Vec<String> = response.json().await?;
    Ok(result)
}

pub async fn get_snapshot_tables(file_name: &str) -> Result<Vec<SnapshotTableApiModel>, RequestError> {
    let url = format!("{}/api/Backup/Tables", get_base_url());
    let response = reqwest::Client::new()
        .get(&url)
        .query(&[("fileName", file_name)])
        .send()
        .await?;
    if !response.status().is_success() {
        return Err(RequestError {
            message: format!("Failed to load snapshot tables: {}", response.status()),
        });
    }
    let result: Vec<SnapshotTableApiModel> = response.json().await?;
    Ok(result)
}

pub async fn get_snapshot_partitions(
    file_name: &str,
    table_name: &str,
) -> Result<Vec<String>, RequestError> {
    let url = format!("{}/api/Backup/Partitions", get_base_url());
    let response = reqwest::Client::new()
        .get(&url)
        .query(&[("fileName", file_name), ("tableName", table_name)])
        .send()
        .await?;
    if !response.status().is_success() {
        return Err(RequestError {
            message: format!("Failed to load snapshot partitions: {}", response.status()),
        });
    }
    let result: Vec<String> = response.json().await?;
    Ok(result)
}

pub async fn get_snapshot_rows(
    file_name: &str,
    table_name: &str,
    partition_key: &str,
) -> Result<Vec<Value>, RequestError> {
    let url = format!("{}/api/Backup/Rows", get_base_url());
    let response = reqwest::Client::new()
        .get(&url)
        .query(&[
            ("fileName", file_name),
            ("tableName", table_name),
            ("partitionKey", partition_key),
        ])
        .send()
        .await?;
    if !response.status().is_success() {
        return Err(RequestError {
            message: format!("Failed to load snapshot rows: {}", response.status()),
        });
    }
    let result: Vec<Value> = response.json().await?;
    Ok(result)
}

pub async fn bulk_delete_rows(
    table_name: &str,
    partition_key: &str,
    row_keys: &[String],
) -> Result<(), RequestError> {
    let mut body = std::collections::BTreeMap::new();
    body.insert(partition_key.to_string(), row_keys.to_vec());

    let url = format!("{}/api/Bulk/Delete", get_base_url());
    let response = reqwest::Client::new()
        .post(&url)
        .query(&[("tableName", table_name)])
        .json(&body)
        .send()
        .await?;
    if !response.status().is_success() {
        return Err(RequestError {
            message: format!("Failed to bulk-delete rows: {}", response.status()),
        });
    }
    Ok(())
}
