use std::{collections::BTreeMap, sync::Arc};

use rust_extensions::date_time::DateTimeAsMicroseconds;
use tokio::sync::RwLock;

use crate::{
    db::{DbPartitionSnapshot, DbRow},
    json::JsonArrayBuilder,
};

use super::{db_table_attributes::DbTableAttributes, db_table_data::DbTableData};

pub struct DbTable {
    pub name: String,
    pub created: DateTimeAsMicroseconds,
    pub data: RwLock<DbTableData>,
}

pub struct DbTableMetrics {
    pub table_size: usize,
    pub partitions_amount: usize,
}

impl DbTable {
    pub fn new(data: DbTableData, created: DateTimeAsMicroseconds) -> Self {
        DbTable {
            created,
            name: data.name.to_string(),
            data: RwLock::new(data),
        }
    }

    pub async fn get_metrics(&self) -> DbTableMetrics {
        let read_access = self.data.read().await;

        return DbTableMetrics {
            table_size: read_access.table_size,
            partitions_amount: read_access.get_partitions_amount(),
        };
    }

    pub async fn get_attributes(&self) -> DbTableAttributes {
        let read_access = self.data.read().await;

        return read_access.attributes.clone();
    }

    pub async fn get_partitions_amount(&self) -> usize {
        let read_access = self.data.read().await;
        return read_access.get_partitions_amount();
    }

    pub async fn set_table_attributes(
        &self,
        persist_table: bool,
        max_partitions_amount: Option<usize>,
    ) -> bool {
        let mut write_access = self.data.write().await;
        if write_access.attributes.persist == persist_table
            && write_access.attributes.max_partitions_amount == max_partitions_amount
        {
            return false;
        }

        write_access.attributes.persist = persist_table;
        write_access.attributes.max_partitions_amount = max_partitions_amount;

        return true;
    }

    pub async fn get_persist(&self) -> bool {
        let table_data = self.data.read().await;
        return table_data.attributes.persist;
    }

    pub async fn as_json(&self) -> Vec<u8> {
        let mut result = JsonArrayBuilder::new();
        let read_access = self.data.read().await;

        for db_row in read_access.iterate_all_rows() {
            result.append_json_object(&db_row.data);
        }

        result.build()
    }

    pub async fn get_snapshot_as_partitions(&self) -> BTreeMap<String, DbPartitionSnapshot> {
        let read_access = self.data.read().await;
        read_access.get_snapshot_as_partitions()
    }

    pub async fn get_partition_snapshot(&self, partition_key: &str) -> Option<DbPartitionSnapshot> {
        let read_access = self.data.read().await;
        read_access.get_partition_snapshot(partition_key)
    }

    pub async fn get_expired_rows(&self, now: DateTimeAsMicroseconds) -> Option<Vec<Arc<DbRow>>> {
        let mut write_access = self.data.write().await;
        write_access.get_expired_rows_up_to(now)
    }
}
