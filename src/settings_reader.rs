use my_azure_storage_sdk::AzureStorageConnection;
use my_no_sql_sdk::core::rust_extensions::StrOrString;
use serde::{Deserialize, Serialize};
use std::{env, sync::Arc};
use tokio::{fs::File, io::AsyncReadExt};

use crate::{persist_io::PersistIoOperations, sqlite_repo::SqlLiteRepo};

#[derive(Serialize, Deserialize, Debug)]
pub struct SettingsModel {
    #[serde(rename = "PersistenceDest")]
    pub persistence_dest: String,

    #[serde(rename = "Location")]
    pub location: String,

    #[serde(rename = "CompressData")]
    pub compress_data: bool,

    #[serde(rename = "TableApiKey")]
    pub table_api_key: String,

    #[serde(rename = "SkipBrokenPartitions")]
    pub skip_broken_partitions: bool,

    #[serde(rename = "InitThreadsAmount")]
    pub init_threads_amount: usize,

    #[serde(rename = "TcpSendTimeoutSec")]
    pub tcp_send_time_out: u64,

    #[serde(rename = "BackupFolder")]
    backup_folder: String,

    #[serde(rename = "BackupIntervalHours")]
    pub backup_interval_hours: u64,

    #[serde(rename = "MaxBackupsToKeep")]
    pub max_backups_to_keep: usize,

    #[serde(rename = "AutoCreateTableOnReaderSubscribe")]
    pub auto_create_table_on_reader_subscribe: bool,

    #[serde(rename = "InitFromOtherServerUrl")]
    pub init_from_other_server_url: Option<String>,
}

impl SettingsModel {
    pub async fn get_persist_io(&self) -> PersistIoOperations {
        if self.persistence_dest.as_str().ends_with(".sqlite") {
            let file = my_no_sql_server_core::rust_extensions::file_utils::format_path(
                self.persistence_dest.as_str(),
            );
            let sqlite_repo = SqlLiteRepo::new(file.to_string()).await;
            return PersistIoOperations::as_sqlite(sqlite_repo);
        }
        let conn_string = AzureStorageConnection::from_conn_string(self.persistence_dest.as_str());
        PersistIoOperations::as_azure_connection(Arc::new(conn_string))
    }

    pub fn get_backup_folder<'s>(&'s self) -> StrOrString<'s> {
        my_no_sql_sdk::core::rust_extensions::file_utils::format_path(self.backup_folder.as_str())
    }

    pub fn get_init_from_other_server_url<'s>(&'s self) -> Option<&str> {
        if let Some(url) = &self.init_from_other_server_url {
            return Some(url.as_str());
        }

        None
    }
}

pub async fn read_settings() -> SettingsModel {
    let file_name = get_settings_filename();

    let mut file = File::open(file_name).await.unwrap();

    let mut file_content: Vec<u8> = vec![];
    file.read_to_end(&mut file_content).await.unwrap();

    let result: SettingsModel = serde_yaml::from_slice(file_content.as_slice()).unwrap();

    result
}

fn get_settings_filename() -> String {
    let path = env!("HOME");

    if path.ends_with('/') {
        return format!("{}{}", path, ".mynosqlserver");
    }

    return format!("{}{}", path, "/.mynosqlserver");
}
