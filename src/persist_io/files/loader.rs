use tokio::sync::Mutex;

use crate::persist_io::{
    serializers::{self, table_attrs::TableMetadataFileContract},
    TableLoadItem,
};

pub struct TableLoaderFromFile {
    files: Mutex<Vec<String>>,
    path_separator: char,
}

impl TableLoaderFromFile {
    pub async fn new(full_path: String, path_separator: char) -> Self {
        let files = get_files(full_path.as_str()).await;
        Self {
            files: Mutex::new(files),
            path_separator,
        }
    }

    async fn get_next_file_name(&self) -> Option<String> {
        let mut access = self.files.lock().await;
        if access.len() == 0 {
            return None;
        }

        let blob_name = access.remove(0);
        Some(blob_name)
    }

    pub async fn get_next(&self) -> Option<TableLoadItem> {
        let full_file_name = self.get_next_file_name().await?;

        let file_name = extract_file_name(full_file_name.as_str(), self.path_separator);

        let content = tokio::fs::read_to_string(full_file_name.as_str())
            .await
            .unwrap();

        if file_name == serializers::table_attrs::METADATA_FILE_NAME {
            let table_metadata = TableMetadataFileContract::parse(content.as_bytes());

            return TableLoadItem::TableAttributes(table_metadata.into()).into();
        }

        let partition_key = serializers::blob_file_name::decode(file_name);

        TableLoadItem::DbPartition {
            partition_key,
            db_partition: serializers::db_partition::deserialize(content.as_bytes()),
        }
        .into()
    }
}

async fn get_files(full_path: &str) -> Vec<String> {
    let mut result = Vec::new();
    for entry in std::fs::read_dir(full_path).unwrap() {
        if let Ok(entity) = entry {
            let file_type = entity.file_type().unwrap();

            if file_type.is_file() {
                let path = entity.path();
                let file_name = format!("{}", path.display());
                result.push(file_name);
            }
        }
    }

    result
}

pub fn extract_file_name(full_path: &str, separator: char) -> &str {
    let full_path_as_bytes = full_path.as_bytes();

    for index in (0..full_path_as_bytes.len()).rev() {
        if full_path_as_bytes[index] == separator as u8 {
            return &full_path[index + 1..];
        }
    }

    panic!("Can not extract filename from fullpath [{}]", full_path);
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_extract_file_name() {
        let src_path = "/Users/Folder/FileName";
        let result = extract_file_name(src_path, '/');
        assert_eq!("FileName", result);
    }
}
