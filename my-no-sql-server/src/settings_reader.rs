use my_no_sql_sdk::core::rust_extensions;

use serde::{Deserialize, Serialize};

use crate::files_repo::FilesRepo;
use crate::persist_repo::PersistRepo;
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
    /// Selects the persistence backend from `PersistenceDest`: a path ending in
    /// `.sqlite` / `.sqlite3` / `.db` uses the SQLite backend, any other path is
    /// treated as a directory and uses the slotted-page `FilesRepo`.
    pub async fn get_persist_repo(&self) -> PersistRepo {
        let dest = my_no_sql_sdk::server::rust_extensions::file_utils::format_path(
            self.persistence_dest.as_str(),
        )
        .to_string();

        let lower = dest.to_lowercase();
        if lower.ends_with(".sqlite") || lower.ends_with(".sqlite3") || lower.ends_with(".db") {
            PersistRepo::Sqlite(SqlLiteRepo::new(dest).await)
        } else {
            PersistRepo::Files(FilesRepo::open(dest, self.skip_broken_partitions).await)
        }
    }

    pub fn get_backup_folder<'s>(&'s self) -> rust_extensions::StrOrString<'s> {
        rust_extensions::file_utils::format_path(self.backup_folder.as_str())
    }

    pub fn get_init_from_other_server_url(&self) -> Option<&str> {
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
