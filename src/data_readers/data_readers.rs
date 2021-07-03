use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;

use super::{data_reader::DataReader, data_reader_contract::DataReaderContract};

pub struct DataReaders {
    data_readers: RwLock<HashMap<u64, Arc<DataReader>>>,
}

impl DataReaders {
    pub fn new() -> Self {
        Self {
            data_readers: RwLock::new(HashMap::new()),
        }
    }

    pub async fn add(&self, data_reader: Arc<DataReader>) {
        let mut write_access = self.data_readers.write().await;
        write_access.insert(data_reader.id, data_reader);
    }

    pub async fn disconnect(&self, id: u64) {
        let mut write_access = self.data_readers.write().await;
        let removed_item = write_access.remove(&id);

        if let Some(data_reader) = removed_item {
            data_reader.disconnect().await;
        }
    }

    pub async fn get(&self, id: &u64) -> Option<Arc<DataReader>> {
        let read_access = self.data_readers.read().await;

        let result = read_access.get(id)?;

        return Some(result.clone());
    }

    pub async fn get_all(&self) -> Vec<Arc<DataReader>> {
        let read_access = self.data_readers.read().await;

        let mut result = Vec::new();

        for reader in read_access.values() {
            result.push(reader.clone());
        }

        result
    }

    pub async fn broadcast(&self, contract: DataReaderContract) {
        let payload = contract.serialize();
        let read_access = self.data_readers.read().await;

        for data_reader in read_access.values() {
            data_reader
                .send_package(contract.get_table_name(), payload.as_slice())
                .await;
        }
    }
}
