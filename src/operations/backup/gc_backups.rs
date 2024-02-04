use my_no_sql_sdk::core::rust_extensions::ShortString;

use crate::app::AppContext;

use std::{collections::BTreeMap, path::MAIN_SEPARATOR};

use super::utils::compile_backup_file;

pub async fn gc_backups(app: &AppContext) {
    let backup_folder = app.settings.get_backup_folder();

    let mut read_dir = tokio::fs::read_dir(backup_folder.as_str()).await.unwrap();

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

            let file_name = extract_file_name(path.as_str(), MAIN_SEPARATOR);

            if if_filename_is_backup(file_name) {
                result.insert(file_name.to_string(), ());
            }
        }
    }

    while result.len() > app.settings.max_backups_to_keep {
        let file_name: ShortString = result.keys().next().unwrap().into();
        println!("Deleting backup file: {}", file_name.as_str());
        delete_backup(app, file_name.as_str()).await;
        result.remove(file_name.as_str());
    }
}

async fn delete_backup(app: &AppContext, file_name: &str) {
    let file_full_path = compile_backup_file(app, file_name);
    tokio::fs::remove_file(file_full_path).await.unwrap();
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
