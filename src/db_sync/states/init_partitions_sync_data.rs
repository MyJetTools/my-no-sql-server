use std::collections::BTreeMap;

use my_json::json_writer::{EmptyJsonArray, JsonObjectWriter};
use my_no_sql_sdk::core::db::DbTable;
use my_no_sql_server_core::db_snapshots::DbPartitionSnapshot;

use crate::db_sync::EventSource;

use super::SyncTableData;

pub struct InitPartitionsSyncData {
    pub table_data: SyncTableData,
    pub event_src: EventSource,
    pub partitions_to_update: BTreeMap<String, Option<DbPartitionSnapshot>>,
}

impl InitPartitionsSyncData {
    pub fn new(table_data: &DbTable, event_src: EventSource) -> Self {
        Self {
            table_data: SyncTableData::new(table_data),
            event_src,
            partitions_to_update: BTreeMap::new(),
        }
    }

    pub fn new_as_update_partition(
        db_table: &DbTable,
        partition_key: &str,
        event_src: EventSource,
    ) -> Self {
        let mut partitions_to_update = BTreeMap::new();

        if let Some(db_partition) = db_table.get_partition(partition_key) {
            let partition_snapshot: DbPartitionSnapshot = db_partition.into();
            partitions_to_update.insert(partition_key.to_string(), Some(partition_snapshot));
        } else {
            partitions_to_update.insert(partition_key.to_string(), None);
        }

        Self {
            table_data: SyncTableData::new(db_table),
            event_src,
            partitions_to_update,
        }
    }

    pub fn add(&mut self, partition_key: String, snapshot: Option<DbPartitionSnapshot>) {
        self.partitions_to_update.insert(partition_key, snapshot);
    }

    pub fn as_json(&self) -> JsonObjectWriter {
        let mut json_object_writer = JsonObjectWriter::new();

        for (partition_key, db_partition) in &self.partitions_to_update {
            if let Some(db_partition_snapshot) = db_partition {
                json_object_writer.write_object(
                    partition_key,
                    db_partition_snapshot.db_rows_snapshot.as_json_array(),
                );
            } else {
                json_object_writer.write_value(partition_key, EmptyJsonArray)
            }
        }

        json_object_writer
    }
}
