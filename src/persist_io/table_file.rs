use my_no_sql_sdk::core::db::PartitionKey;
use rust_extensions::base64::FromBase64;

use crate::persist_io::TABLE_METADATA_FILE_NAME;

pub enum TableFile {
    TableAttributes,
    DbPartition(PartitionKey),
}

pub struct TableFileName<'s> {
    as_str: Option<&'s str>,
    as_string: Option<String>,
}

impl<'s> TableFileName<'s> {
    pub fn new(as_str: &'s str) -> Self {
        Self {
            as_str: Some(as_str),
            as_string: None,
        }
    }

    pub fn new_as_string(as_string: String) -> Self {
        Self {
            as_string: Some(as_string),
            as_str: None,
        }
    }

    pub fn as_str(&'s self) -> &'s str {
        if let Some(as_str) = self.as_str {
            return as_str;
        }

        if let Some(as_string) = &self.as_string {
            return as_string;
        }

        panic!("TableFileName is not initialized properly");
    }
}

impl TableFile {
    pub fn from_file_name(file_name: &str) -> Result<Self, String> {
        if file_name == TABLE_METADATA_FILE_NAME {
            return Ok(Self::TableAttributes);
        }

        let partition_key = file_name.from_base64();

        if partition_key.is_err() {
            return Err(format!(
                "Can not decode filename: {}. Err:{:?}",
                file_name,
                partition_key.err()
            ));
        }

        let partition_key = partition_key.unwrap();

        match String::from_utf8(partition_key) {
            Ok(result) => Ok(Self::DbPartition(result.into())),
            Err(err) => Err(format!(
                "Can not decode filename: {}. Err:{:?}",
                file_name, err
            )),
        }
    }
    pub fn get_file_name(&self) -> TableFileName {
        match self {
            TableFile::TableAttributes => TableFileName::new(TABLE_METADATA_FILE_NAME),
            TableFile::DbPartition(partition_key) => {
                use base64::Engine;
                let encoded = base64::engine::general_purpose::STANDARD
                    .encode(partition_key.as_str().as_bytes());
                TableFileName::new_as_string(encoded)
            }
        }
    }
}
