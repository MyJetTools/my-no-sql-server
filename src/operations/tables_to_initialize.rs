use tokio::sync::Mutex;

pub struct TablesToInitialize {
    tables: Mutex<Vec<String>>,
}

impl TablesToInitialize {
    pub fn new(tables: Vec<String>) -> Self {
        Self {
            tables: Mutex::new(tables),
        }
    }

    pub async fn get(&self) -> Option<String> {
        let mut data_access = self.tables.lock().await;

        if data_access.len() == 0 {
            return None;
        }

        let result = data_access.remove(0);
        return Some(result);
    }
}
