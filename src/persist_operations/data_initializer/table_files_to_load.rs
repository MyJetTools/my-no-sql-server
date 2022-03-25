use tokio::sync::Mutex;

use crate::persist_io::TableFile;

pub struct TableFilesToLoad {
    partitions: Mutex<Vec<TableFile>>,
}

impl<'s> TableFilesToLoad {
    pub fn new(partitions: Vec<TableFile>) -> Self {
        Self {
            partitions: Mutex::new(partitions),
        }
    }

    pub async fn get_next(&'s self) -> Option<TableFile> {
        let mut write_access = self.partitions.lock().await;

        if write_access.len() == 0 {
            return None;
        }

        let result = write_access.remove(0);
        Some(result)
    }
}
