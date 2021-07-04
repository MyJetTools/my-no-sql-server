use std::{collections::HashMap, sync::Arc, time::Duration};

use tokio::sync::RwLock;

use crate::{date_time::MyDateTime, utils::ItemsOrNone};

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

    async fn get_items_to_gc(
        &self,
        now: MyDateTime,
        max_inactive_duration: Duration,
    ) -> ItemsOrNone<u64> {
        let read_access = self.data_readers.read().await;

        let mut items_to_gc = ItemsOrNone::new();

        for data_reader in read_access.values() {
            let inactive_duration = data_reader.last_incoming_package.duration_to(now);

            if inactive_duration.is_none() {
                continue;
            }

            let inactive_duration = inactive_duration.unwrap();

            if inactive_duration < max_inactive_duration {
                continue;
            }

            items_to_gc.push(data_reader.id);
        }

        return items_to_gc;
    }

    pub async fn gc(&self, now: MyDateTime, max_inactive_duration: Duration) {
        let connections_to_gc = self.get_items_to_gc(now, max_inactive_duration).await;

        let connections_to_gc = connections_to_gc.get();

        if connections_to_gc.is_none() {
            return;
        }

        let connections_to_gc = connections_to_gc.unwrap();

        let mut write_access = self.data_readers.write().await;
        for id in connections_to_gc {
            let remove_connection_result = write_access.remove(id);

            if let Some(connection) = remove_connection_result {
                connection.disconnect().await;
            }
        }
    }
}
