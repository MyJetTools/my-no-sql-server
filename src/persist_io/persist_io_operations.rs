pub const TABLE_METADATA_FILE_NAME: &str = ".metadata";

pub enum TableFile {
    TableAttributes,
    DbPartition(String),
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

        panic!("TableFileName is not initialized propertly");
    }
}

impl TableFile {
    pub fn from_file_name(file_name: &str) -> Result<Self, String> {
        if file_name == TABLE_METADATA_FILE_NAME {
            return Ok(Self::TableAttributes);
        }

        let partition_key = base64::decode(file_name).unwrap();

        match String::from_utf8(partition_key) {
            Ok(result) => Ok(Self::DbPartition(result)),
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
                TableFileName::new_as_string(base64::encode(partition_key.as_bytes()))
            }
        }
    }
}

#[async_trait::async_trait]
pub trait PersistIoOperations {
    async fn get_list_of_tables(&self) -> Vec<String>;

    async fn get_table_files(&self, table_name: &str) -> Vec<TableFile>;

    async fn load_table_file(&self, table_name: &str, table_file: &TableFile) -> Option<Vec<u8>>;

    async fn create_table_folder(&self, table_name: &str);
    async fn delete_table_folder(&self, table_name: &str);

    async fn save_table_file(&self, table_name: &str, table_file: &TableFile, content: Vec<u8>);
    async fn delete_table_file(&self, table_name: &str, table_file: &TableFile);
}
