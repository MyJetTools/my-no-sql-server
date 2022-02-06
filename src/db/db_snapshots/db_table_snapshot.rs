use std::collections::BTreeMap;

use my_json::json_writer::JsonArrayWriter;
use my_no_sql_tcp_shared::TcpContract;
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::db::{DbTableAttributesSnapshot, DbTableData};

use super::DbPartitionSnapshot;

pub struct DbTableSnapshot {
    pub attr: DbTableAttributesSnapshot,
    pub created: DateTimeAsMicroseconds,
    pub last_update: DateTimeAsMicroseconds,
    pub by_partition: BTreeMap<String, DbPartitionSnapshot>,
}

impl DbTableSnapshot {
    pub fn new(table_data: &DbTableData, attr: DbTableAttributesSnapshot) -> Self {
        let mut by_partition = BTreeMap::new();

        for (partition_key, db_partition) in &table_data.partitions {
            by_partition.insert(partition_key.to_string(), db_partition.into());
        }

        Self {
            attr,
            created: table_data.created,
            last_update: table_data.last_update_time.as_date_time(),
            by_partition,
        }
    }

    pub fn into_tcp_contract(&self, table_name: String) -> TcpContract {
        let data = self.as_json_array().build();
        TcpContract::InitTable { table_name, data }
    }

    pub fn as_json_array(&self) -> JsonArrayWriter {
        let mut json_array_writer = JsonArrayWriter::new();

        for db_partition_snapshot in self.by_partition.values() {
            json_array_writer.write_object(db_partition_snapshot.db_rows.as_json_array());
        }

        json_array_writer
    }
}
