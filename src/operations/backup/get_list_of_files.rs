use std::collections::BTreeMap;

use serde_derive::Serialize;

use std::sync::Arc;

use crate::app::{AppContext, DbNamespace};

#[derive(Serialize)]
pub struct SnapshotFileModel {
    pub name: String,
    pub size: i64,
}

pub async fn get_list_of_files(
    app: &AppContext,
    db_namespace: &Arc<DbNamespace>,
) -> Vec<SnapshotFileModel> {
    let backup_folder = super::utils::get_backup_folder(app, &db_namespace.name);

    // A namespace which was never backed up simply has no folder yet.
    let mut read_dir = match tokio::fs::read_dir(backup_folder.as_str()).await {
        Ok(read_dir) => read_dir,
        Err(_) => return Vec::new(),
    };

    let mut result = BTreeMap::new();

    while let Ok(entry) = read_dir.next_entry().await {
        if entry.is_none() {
            break;
        }

        let entry = entry.unwrap();

        let file_type = entry.file_type().await.unwrap();

        if file_type.is_file() {
            let path = entry.path();

            let path = format!("{}", path.display());

            let file_name = extract_file_name(path.as_str(), std::path::MAIN_SEPARATOR);

            if if_filename_is_backup(file_name) {
                let size = entry.metadata().await.map(|m| m.len()).unwrap_or(0);
                result.insert(file_name.to_string(), size as i64);
            }
        }
    }

    result
        .into_iter()
        .map(|(name, size)| SnapshotFileModel { name, size })
        .collect()
}

pub fn extract_file_name(full_path: &str, separator: char) -> &str {
    let full_path_as_bytes = full_path.as_bytes();

    for index in (0..full_path_as_bytes.len()).rev() {
        if full_path_as_bytes[index] == separator as u8 {
            return &full_path[index + 1..];
        }
    }

    panic!("Can not extract filename from full path [{}]", full_path);
}

fn if_filename_is_backup(src: &str) -> bool {
    return src.ends_with(".zip");
}
