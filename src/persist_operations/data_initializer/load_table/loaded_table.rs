use tokio::sync::Mutex;

use super::LoadedTableItem;

pub struct LoadedTable {
    items: Mutex<Vec<LoadedTableItem>>,
}

impl LoadedTable {
    pub fn new() -> Self {
        Self {
            items: Mutex::new(Vec::new()),
        }
    }

    pub async fn add(&self, item: LoadedTableItem) -> usize {
        let mut write_access = self.items.lock().await;
        write_access.push(item);
        write_access.len()
    }

    pub async fn get(&self) -> Vec<LoadedTableItem> {
        let mut result = Vec::new();
        let mut write_access = self.items.lock().await;

        std::mem::swap(&mut *write_access, &mut result);

        return result;
    }
}
