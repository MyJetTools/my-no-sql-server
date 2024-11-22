use my_no_sql_sdk::core::rust_extensions::StrOrString;
use my_no_sql_server_core::rust_extensions;
use serde::{Deserialize, Serialize};

use crate::sqlite_repo::SqlLiteRepo;

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
    pub async fn get_sqlite_repo(&self) -> SqlLiteRepo {
        let file = my_no_sql_server_core::rust_extensions::file_utils::format_path(
            self.persistence_dest.as_str(),
        );
        SqlLiteRepo::new(file.to_string()).await
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
    let file_name = rust_extensions::file_utils::format_path("~/.mynosqlserver");

    let file_content = tokio::fs::read(file_name.as_str()).await;

    if let Err(err) = &file_content {
        panic!(
            "Can't open settings file [{}]. Err: {}",
            file_name.as_str(),
            err
        );
    }

    let file_content = file_content.unwrap();

    let result: SettingsModel = serde_yaml::from_slice(file_content.as_slice()).unwrap();

    result
}

/*
fn get_settings_filename() -> String {
    let path = env!("HOME");

    if path.ends_with('/') {
        return format!("{}{}", path, ".mynosqlserver");
    }

    return format!("{}{}", path, "/.mynosqlserver");
}
 */
