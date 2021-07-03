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
}

impl SettingsModel {
    pub fn persist_to_blob(&self) -> bool {
        return !self.persistence_dest.starts_with("http");
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
