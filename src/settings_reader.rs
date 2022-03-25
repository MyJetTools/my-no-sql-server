use my_app_insights::AppInsightsTelemetry;
use my_azure_storage_sdk::AzureStorageConnection;
use serde::{Deserialize, Serialize};
use std::{env, sync::Arc};
use tokio::{fs::File, io::AsyncReadExt};

use crate::{app::logs::Logs, persist_io::AzureBlobsPersistIo};

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
    #[serde(rename = "InitTablesThreadsAmount")]
    pub init_tabes_thread: usize,
    #[serde(rename = "InitThreadsAmount")]
    pub init_threads_amount: usize,
}

impl SettingsModel {
    pub fn persist_to_blob(&self) -> bool {
        return self
            .persistence_dest
            .starts_with("DefaultEndpointsProtocol");
    }

    pub fn get_persist_io(
        &self,
        logs: Arc<Logs>,
        telemetry: Arc<AppInsightsTelemetry>,
    ) -> AzureBlobsPersistIo {
        let mut conn_string =
            AzureStorageConnection::from_conn_string(self.persistence_dest.as_str());

        conn_string.set_telemetry(telemetry);

        AzureBlobsPersistIo::new(Arc::new(conn_string), logs)
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
