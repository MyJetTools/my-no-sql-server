use std::collections::BTreeMap;

use my_no_sql_sdk::server::rust_extensions::base64::FromBase64;
use serde_derive::Serialize;

use crate::{app::AppContext, scripts::TABLE_METADATA_FILE_NAME, zip::ZipReader};

#[derive(Debug)]
pub enum InspectError {
    InvalidFileName,
    FileNotFound,
    IoError(String),
    TableNotFound,
    PartitionNotFound,
    InvalidPartitionKey,
}

impl InspectError {
    pub fn into_message(self) -> String {
        match self {
            InspectError::InvalidFileName => "Invalid snapshot file name".to_string(),
            InspectError::FileNotFound => "Snapshot file not found".to_string(),
            InspectError::IoError(s) => format!("I/O error: {}", s),
            InspectError::TableNotFound => "Table not found in snapshot".to_string(),
            InspectError::PartitionNotFound => "Partition not found in snapshot".to_string(),
            InspectError::InvalidPartitionKey => "Invalid partition key encoding".to_string(),
        }
    }
}

fn validate_file_name(file_name: &str) -> Result<(), InspectError> {
    if file_name.is_empty()
        || file_name.contains('/')
        || file_name.contains('\\')
        || file_name.contains("..")
        || !file_name.ends_with(".zip")
    {
        return Err(InspectError::InvalidFileName);
    }
    Ok(())
}

async fn load_zip(app: &AppContext, file_name: &str) -> Result<ZipReader, InspectError> {
    validate_file_name(file_name)?;
    let full_path = super::utils::compile_backup_file(app, file_name);
    let content = tokio::fs::read(full_path.as_str())
        .await
        .map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => InspectError::FileNotFound,
            _ => InspectError::IoError(e.to_string()),
        })?;
    Ok(ZipReader::new(content))
}

#[derive(Serialize)]
pub struct SnapshotTable {
    pub name: String,
    #[serde(rename = "partitionsCount")]
    pub partitions_count: usize,
}

pub async fn list_snapshot_tables(
    app: &AppContext,
    file_name: &str,
) -> Result<Vec<SnapshotTable>, InspectError> {
    let mut zip = load_zip(app, file_name).await?;

    let mut counts: BTreeMap<String, usize> = BTreeMap::new();

    for entry in zip.get_file_names() {
        let Some(idx) = entry.find('/') else { continue };
        let table = &entry[..idx];
        let rest = &entry[idx + 1..];
        if rest.is_empty() {
            continue;
        }
        let bump = if rest == TABLE_METADATA_FILE_NAME {
            0
        } else {
            1
        };
        *counts.entry(table.to_string()).or_insert(0) += bump;
    }

    Ok(counts
        .into_iter()
        .map(|(name, partitions_count)| SnapshotTable {
            name,
            partitions_count,
        })
        .collect())
}

pub async fn list_snapshot_partitions(
    app: &AppContext,
    file_name: &str,
    table_name: &str,
) -> Result<Vec<String>, InspectError> {
    let mut zip = load_zip(app, file_name).await?;

    let prefix = format!("{}/", table_name);
    let mut found_table = false;
    let mut partitions: Vec<String> = Vec::new();

    for entry in zip.get_file_names() {
        if !entry.starts_with(&prefix) {
            continue;
        }
        found_table = true;
        let rest = &entry[prefix.len()..];
        if rest.is_empty() || rest == TABLE_METADATA_FILE_NAME {
            continue;
        }
        let bytes = rest
            .from_base64()
            .map_err(|_| InspectError::InvalidPartitionKey)?;
        let pk = String::from_utf8(bytes).map_err(|_| InspectError::InvalidPartitionKey)?;
        partitions.push(pk);
    }

    if !found_table {
        return Err(InspectError::TableNotFound);
    }

    partitions.sort();
    Ok(partitions)
}

pub async fn read_snapshot_partition_rows(
    app: &AppContext,
    file_name: &str,
    table_name: &str,
    partition_key: &str,
) -> Result<Vec<u8>, InspectError> {
    use base64::Engine;
    let encoded = base64::engine::general_purpose::STANDARD.encode(partition_key.as_bytes());
    let zip_path = format!("{}/{}", table_name, encoded);

    let mut zip = load_zip(app, file_name).await?;
    zip.get_content_as_vec(&zip_path)
        .map_err(|_| InspectError::PartitionNotFound)
}
