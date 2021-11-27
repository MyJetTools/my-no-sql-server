use my_app_insights::AppInsightsTelemetry;
use my_azure_storage_sdk::AzureConnectionWithTelemetry;
use serde::{Deserialize, Serialize};
use std::env;
use tokio::{fs::File, io::AsyncReadExt};

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

    #[serde(rename = "InitThreadsAmount")]
    pub init_threads_amount: usize,
}

impl SettingsModel {
    pub fn persist_to_blob(&self) -> bool {
        return !self.persistence_dest.starts_with("http");
    }

    pub fn get_azure_connection(
        &self,
    ) -> Option<AzureConnectionWithTelemetry<AppInsightsTelemetry>> {
        if !self.persist_to_blob() {
            return None;
        }
        let result =
            AzureConnectionWithTelemetry::from_conn_string(self.persistence_dest.as_str(), None);
        return Some(result);
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

    if path.ends_with("/") {
        return format!("{}{}", path, ".mynosqlserver");
    }

    return format!("{}{}", path, "/.mynosqlserver");
}
