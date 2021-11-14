use std::collections::BTreeMap;

use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{db::DbPartitionSnapshot, json::JsonArrayBuilder};

use super::{db_table_attributes::DbTableAttributes, DbTableData};

pub struct DbTableSnapshot {
    pub attr: DbTableAttributes,
    pub created: DateTimeAsMicroseconds,
    pub last_update: DateTimeAsMicroseconds,
    pub data: BTreeMap<String, DbPartitionSnapshot>,
}

impl DbTableSnapshot {
    pub fn new(table_data: &DbTableData) -> Self {
        let data = table_data.get_snapshot_as_partitions();

        Self {
            attr: table_data.attributes.clone(),
            created: table_data.created,
            last_update: table_data.last_update_time.as_date_time(),
            data,
        }
    }

    pub fn as_raw_bytes(&self) -> Vec<u8> {
        let mut result = JsonArrayBuilder::new();

        for db_partition in self.data.values() {
            for db_row in &db_partition.content {
                result.append_json_object(&db_row.data);
            }
        }

        return result.build();
    }
}
