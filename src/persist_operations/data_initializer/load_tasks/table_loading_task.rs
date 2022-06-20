use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use rust_extensions::date_time::DateTimeAsMicroseconds;
use tokio::sync::Mutex;

use crate::{
    db::{DbTableAttributesSnapshot, DbTableData},
    persist_operations::data_initializer::LoadedTableItem,
};

pub struct TableLoadingTaskData {
    db_table_data: Option<DbTableData>,
    attrs: Option<DbTableAttributesSnapshot>,
}

impl TableLoadingTaskData {
    pub fn new(table_name: String) -> Self {
        Self {
            db_table_data: Some(DbTableData::new(table_name, DateTimeAsMicroseconds::now())),
            attrs: None,
        }
    }
}

pub struct TableLoadingTask {
    pub table_name: String,
    pub db_table_data: Mutex<TableLoadingTaskData>,
    pub file_list_is_loaded: AtomicBool,
    pub total_files_amount: AtomicUsize,
    pub loaded_files_amount: AtomicUsize,
}

impl TableLoadingTask {
    pub fn new(table_name: String) -> Self {
        Self {
            table_name: table_name.to_string(),
            file_list_is_loaded: AtomicBool::new(false),
            db_table_data: Mutex::new(TableLoadingTaskData::new(table_name)),
            total_files_amount: AtomicUsize::new(0),
            loaded_files_amount: AtomicUsize::new(0),
        }
    }

    pub fn add_total_files_amount(&self, amount: usize) {
        self.total_files_amount.fetch_add(amount, Ordering::SeqCst);
    }

    pub fn set_file_list_is_loaded(&self) {
        self.file_list_is_loaded.store(true, Ordering::SeqCst);
    }

    pub fn everything_is_loaded(&self) -> bool {
        let file_list_is_loaded = self.file_list_is_loaded.load(Ordering::SeqCst);
        if !file_list_is_loaded {
            return false;
        }

        let total_files_amount = self.total_files_amount.load(Ordering::SeqCst);
        let loaded_files_amount = self.loaded_files_amount.load(Ordering::SeqCst);

        total_files_amount == loaded_files_amount
    }

    pub async fn add_loaded_file(&self, item: LoadedTableItem) -> bool {
        self.loaded_files_amount.fetch_add(1, Ordering::SeqCst);

        let mut write_access = self.db_table_data.lock().await;

        match item {
            LoadedTableItem::TableAttributes(attrs) => {
                write_access.attrs = Some(attrs.into());
            }
            LoadedTableItem::DbPartition {
                partition_key,
                db_partition,
            } => {
                write_access
                    .db_table_data
                    .as_mut()
                    .unwrap()
                    .partitions
                    .insert(partition_key, db_partition);
            }
        }

        return self.everything_is_loaded();
    }

    pub async fn get_db_table_data(&self) -> (DbTableData, DbTableAttributesSnapshot) {
        let mut write_access = self.db_table_data.lock().await;

        if write_access.db_table_data.is_none() {
            panic!("db_table_data is None");
        }

        let mut db_table_data = None;
        std::mem::swap(&mut db_table_data, &mut write_access.db_table_data);

        let attrs = if let Some(attrs) = &write_access.attrs {
            attrs.clone()
        } else {
            DbTableAttributesSnapshot::create_default()
        };

        (db_table_data.unwrap(), attrs)
    }
}
