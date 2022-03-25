use tokio::sync::Mutex;

use super::table_load_item::TableLoadItem;

pub struct LoadedTable {
    items: Mutex<Vec<TableLoadItem>>,
}

impl LoadedTable {
    pub fn new() -> Self {
        Self {
            items: Mutex::new(Vec::new()),
        }
    }

    pub async fn add(&self, item: TableLoadItem) -> usize {
        let mut write_access = self.items.lock().await;
        write_access.push(item);
        write_access.len()
    }

    pub async fn get(&self) -> Vec<TableLoadItem> {
        let mut result = Vec::new();
        let mut write_access = self.items.lock().await;

        std::mem::swap(&mut *write_access, &mut result);

        return result;
    }
}
