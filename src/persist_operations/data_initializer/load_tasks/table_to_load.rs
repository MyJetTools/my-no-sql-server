use std::sync::atomic::{AtomicBool, AtomicI64, AtomicUsize, Ordering};

use rust_extensions::date_time::DateTimeAsMicroseconds;
use tokio::sync::Mutex;

pub enum PartitionToLoad {
    Load(String),
    EndOfReading,
}

pub struct TableToLoad {
    pub table_name: String,
    partitions: Mutex<Vec<String>>,
    files_to_load: AtomicUsize,
    files_loaded: AtomicUsize,
    files_list_is_loaded: AtomicBool,
    initializing_is_started: AtomicI64,
}

impl TableToLoad {
    pub fn new(table_name: String) -> Self {
        Self {
            partitions: Mutex::new(Vec::new()),
            table_name,
            files_to_load: AtomicUsize::new(0),
            files_loaded: AtomicUsize::new(0),
            files_list_is_loaded: AtomicBool::new(false),
            initializing_is_started: AtomicI64::new(-1),
        }
    }
    pub async fn add_partitions_to_load(&self, partition_keys: Vec<String>) {
        let mut write_access = self.partitions.lock().await;
        write_access.extend(partition_keys)
    }

    pub fn set_files_list_is_loaded(&self) {
        self.files_list_is_loaded.store(true, Ordering::SeqCst);
    }

    pub fn get_files_to_load(&self) -> usize {
        self.files_to_load.load(Ordering::Relaxed)
    }

    pub fn get_files_loaded(&self) -> usize {
        self.files_loaded.load(Ordering::Relaxed)
    }

    pub fn get_files_list_is_loaded(&self) -> bool {
        self.files_list_is_loaded.load(Ordering::Relaxed)
    }

    pub fn get_initializing_is_started(&self) -> Option<DateTimeAsMicroseconds> {
        let result = self.initializing_is_started.load(Ordering::Relaxed);

        if result == -1 {
            return None;
        }

        DateTimeAsMicroseconds::new(result).into()
    }

    pub fn inc_files_loaded(&self) {
        self.initializing_is_started.fetch_add(1, Ordering::Relaxed);
    }

    fn update_initializing_is_started_if_needed(&self) {
        let result = self.initializing_is_started.load(Ordering::Relaxed);
        if result > -1 {
            return;
        }

        let now = DateTimeAsMicroseconds::now();

        self.initializing_is_started
            .store(now.unix_microseconds, Ordering::SeqCst);
    }

    pub async fn get_next(&self) -> Option<PartitionToLoad> {
        self.update_initializing_is_started_if_needed();

        let mut write_access = self.partitions.lock().await;
        if write_access.len() > 0 {
            let result = write_access.remove(0);
            return Some(PartitionToLoad::Load(result));
        }

        let end_of_reading = self.files_list_is_loaded.load(Ordering::SeqCst);

        if end_of_reading {
            return Some(PartitionToLoad::EndOfReading);
        }
        return None;
    }
}
