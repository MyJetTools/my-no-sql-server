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

pub fn download_rows_url(table_name: &str, partition_key: &str) -> String {
    format!(
        "{}/api/Row/Download?tableName={}&partitionKey={}",
        get_base_url(),
        url_escape(table_name),
        url_escape(partition_key),
    )
}

fn url_escape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.as_bytes() {
        let c = *byte;
        let safe = matches!(
            c,
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~'
        );
        if safe {
            out.push(c as char);
        } else {
            out.push_str(&format!("%{:02X}", c));
        }
    }
    out
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

pub async fn get_connections() -> Result<ConnectionsApiModel, RequestError> {
    let url = format!("{}/api/Connections", get_base_url());
    let response = reqwest::Client::new().get(&url).send().await?;
    if !response.status().is_success() {
        return Err(RequestError {
            message: format!("Failed to load connections: {}", response.status()),
        });
    }
    let result: ConnectionsApiModel = response.json().await?;
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
    let url = format!(
        "{}/api/Partitions?tableName={}",
        get_base_url(),
        url_escape(table_name),
    );
    let response = reqwest::Client::new().get(&url).send().await?;
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
    let url = format!(
        "{}/api/Row?tableName={}&partitionKey={}",
        get_base_url(),
        url_escape(table_name),
        url_escape(partition_key),
    );
    let response = reqwest::Client::new()
        .get(&url)
        .header("x-compress", "zstd")
        .send()
        .await?;
    if !response.status().is_success() {
        return Err(RequestError {
            message: format!("Failed to load rows: {}", response.status()),
        });
    }
    let compressed = response
        .headers()
        .get("x-content-encoding")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.to_ascii_lowercase().contains("zstd"))
        .unwrap_or(false);

    if compressed {
        let body = response.bytes().await?;
        let decoded = decode_zstd(body.as_ref())?;
        let result: Vec<Value> = serde_json::from_slice(&decoded).map_err(|e| RequestError {
            message: format!("Failed to parse decompressed rows: {}", e),
        })?;
        Ok(result)
    } else {
        let result: Vec<Value> = response.json().await?;
        Ok(result)
    }
}

fn decode_zstd(bytes: &[u8]) -> Result<Vec<u8>, RequestError> {
    use std::io::Read;
    let mut decoder = ruzstd::decoding::StreamingDecoder::new(bytes).map_err(|e| RequestError {
        message: format!("Failed to start zstd decoder: {}", e),
    })?;
    let mut out = Vec::new();
    decoder.read_to_end(&mut out).map_err(|e| RequestError {
        message: format!("Failed to decode zstd body: {}", e),
    })?;
    Ok(out)
}

pub async fn delete_row(
    table_name: &str,
    partition_key: &str,
    row_key: &str,
) -> Result<(), RequestError> {
    let url = format!(
        "{}/api/Row?tableName={}&partitionKey={}&rowKey={}",
        get_base_url(),
        url_escape(table_name),
        url_escape(partition_key),
        url_escape(row_key),
    );
    let response = reqwest::Client::new().delete(&url).send().await?;
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
        #[serde(rename = "mcpWritesEnabled", default)]
        mcp_writes_enabled: bool,
        #[serde(rename = "mcpWritesRemainingSecs", default)]
        mcp_writes_remaining_secs: Option<u64>,
    }
    let p: Payload = response.json().await?;
    Ok(crate::settings::UiServerSettings {
        thresholds: crate::settings::HealthThresholds {
            warn_ms: p.warn_ms,
            bad_ms: p.bad_ms,
        },
        mcp_writes_enabled: p.mcp_writes_enabled,
        mcp_writes_remaining_secs: p.mcp_writes_remaining_secs,
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

/// Enables or disables the MCP write tools via POST
/// `/api/Settings/McpWrites`. Enabling opens a 10-minute window on the
/// server; disabling closes it immediately.
pub async fn set_mcp_writes(enabled: bool) -> Result<(), RequestError> {
    let url = format!("{}/api/Settings/McpWrites", get_base_url());
    #[derive(serde::Serialize)]
    struct Payload {
        enabled: bool,
    }
    let response = reqwest::Client::new()
        .post(&url)
        .json(&Payload { enabled })
        .send()
        .await?;
    if !response.status().is_success() {
        return Err(RequestError {
            message: format!("Failed to update MCP writes: {}", response.status()),
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
    let url = format!(
        "{}/api/Backup/Tables?fileName={}",
        get_base_url(),
        url_escape(file_name),
    );
    let response = reqwest::Client::new().get(&url).send().await?;
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
    let url = format!(
        "{}/api/Backup/Partitions?fileName={}&tableName={}",
        get_base_url(),
        url_escape(file_name),
        url_escape(table_name),
    );
    let response = reqwest::Client::new().get(&url).send().await?;
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
    let url = format!(
        "{}/api/Backup/Rows?fileName={}&tableName={}&partitionKey={}",
        get_base_url(),
        url_escape(file_name),
        url_escape(table_name),
        url_escape(partition_key),
    );
    let response = reqwest::Client::new().get(&url).send().await?;
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

    let url = format!(
        "{}/api/Bulk/Delete?tableName={}",
        get_base_url(),
        url_escape(table_name),
    );
    let response = reqwest::Client::new()
        .post(&url)
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

pub async fn bulk_delete_many(
    table_name: &str,
    grouped: &std::collections::BTreeMap<String, Vec<String>>,
) -> Result<(), RequestError> {
    let url = format!(
        "{}/api/Bulk/Delete?tableName={}",
        get_base_url(),
        url_escape(table_name),
    );
    let response = reqwest::Client::new()
        .post(&url)
        .json(grouped)
        .send()
        .await?;
    if !response.status().is_success() {
        return Err(RequestError {
            message: format!("Failed to bulk-delete rows: {}", response.status()),
        });
    }
    Ok(())
}
