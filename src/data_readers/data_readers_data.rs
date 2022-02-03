use std::{collections::HashMap, sync::Arc};

use super::{DataReader, DataReaderConnection};

pub struct DataReadersData {
    tcp: HashMap<i32, Arc<DataReader>>,
    all: HashMap<String, Arc<DataReader>>,
}

impl DataReadersData {
    pub fn new() -> Self {
        Self {
            tcp: HashMap::new(),
            all: HashMap::new(),
        }
    }

    pub fn insert(&mut self, data_reader: DataReader) {
        let data_reader = Arc::new(data_reader);
        self.all
            .insert(data_reader.id.to_string(), data_reader.clone());

        match &data_reader.connection {
            DataReaderConnection::Tcp(connection) => {
                self.tcp.insert(connection.get_id(), data_reader);
            }
        }
    }

    pub fn get_tcp(&self, connection_id: i32) -> Option<Arc<DataReader>> {
        let result = self.tcp.get(&connection_id)?;
        result.clone().into()
    }

    pub fn remove_tcp(&mut self, connection_id: i32) {
        if let Some(removed_connection) = self.tcp.remove(&connection_id) {
            self.all.remove(&removed_connection.id);
        }
    }

    pub fn get_all(&self) -> Vec<Arc<DataReader>> {
        self.all.values().map(|itm| itm.clone()).collect()
    }

    pub async fn get_subscribred_to_table(&self, table_name: &str) -> Option<Vec<Arc<DataReader>>> {
        let mut result = None;

        for data_reader in self.all.values() {
            if data_reader.has_table(table_name).await {
                if result.is_none() {
                    result = Some(Vec::new());
                }

                result.as_mut().unwrap().push(data_reader.clone());
            }
        }

        result
    }
}
