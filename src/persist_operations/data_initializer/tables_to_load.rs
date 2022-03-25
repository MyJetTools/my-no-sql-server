use tokio::sync::Mutex;

pub struct TablesToLoad {
    data: Mutex<Vec<String>>,
}

impl TablesToLoad {
    pub fn new(src: Vec<String>) -> Self {
        Self {
            data: Mutex::new(src),
        }
    }

    pub async fn get(&self) -> Option<String> {
        let mut write_access = self.data.lock().await;

        if write_access.len() == 0 {
            return None;
        }

        let result = write_access.remove(0);
        Some(result)
    }
}
