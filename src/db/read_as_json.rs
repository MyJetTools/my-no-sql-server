use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};

use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    db::{DbPartition, DbRow, DbTableData},
    json::JsonArrayBuilder,
};

pub trait DbEntityAsJsonArray {
    fn as_json_array(&self, update_read_time: Option<DateTimeAsMicroseconds>) -> Vec<u8>;
}

impl DbEntityAsJsonArray for [&DbRow] {
    fn as_json_array(&self, update_read_time: Option<DateTimeAsMicroseconds>) -> Vec<u8> {
        let mut json_array_builder = JsonArrayBuilder::new();

        for db_row in self {
            json_array_builder.append_json_object(&db_row.data)
        }

        return json_array_builder.build();
    }
}

impl DbEntityAsJsonArray for DbTableData {
    fn as_json_array(&self, update_read_time: Option<DateTimeAsMicroseconds>) -> Vec<u8> {
        let mut json_array_builder = JsonArrayBuilder::new();

        for (_, db_partition) in &self.partitions {
            if let Some(update_read_time) = update_read_time {
                db_partition.last_read_access.update(update_read_time);
            }

            db_partition.fill_with_json_data(&mut json_array_builder, update_read_time);
        }

        return json_array_builder.build();
    }
}

impl DbEntityAsJsonArray for DbPartition {
    fn as_json_array(&self, update_read_time: Option<DateTimeAsMicroseconds>) -> Vec<u8> {
        if let Some(update_read_time) = update_read_time {
            self.last_read_access.update(update_read_time);
        }

        let mut json_array_builder = JsonArrayBuilder::new();
        self.fill_with_json_data(&mut json_array_builder, update_read_time);
        return json_array_builder.build();
    }
}

impl DbEntityAsJsonArray for Vec<Arc<DbRow>> {
    fn as_json_array(&self, update_read_time: Option<DateTimeAsMicroseconds>) -> Vec<u8> {
        let mut json_array_builder = JsonArrayBuilder::new();

        for db_row in self {
            json_array_builder.append_json_object(&db_row.data);

            if let Some(read_time) = update_read_time {
                db_row.update_last_access(read_time);
            }
        }

        json_array_builder.build()
    }
}

impl DbEntityAsJsonArray for BTreeMap<String, Vec<Arc<DbRow>>> {
    fn as_json_array(&self, update_read_time: Option<DateTimeAsMicroseconds>) -> Vec<u8> {
        let mut json_array_builder = JsonArrayBuilder::new();

        for (partition_key, db_partition) in self {
            for db_row in db_partition {
                json_array_builder.append_json_object(&db_row.data);

                if let Some(read_time) = update_read_time {
                    db_row.update_last_access(read_time);
                }
            }
        }

        json_array_builder.build()
    }
}

impl DbEntityAsJsonArray for DbRow {
    fn as_json_array(&self, update_read_time: Option<DateTimeAsMicroseconds>) -> Vec<u8> {
        let mut json_array_builder = JsonArrayBuilder::new();

        json_array_builder.append_json_object(self.data.as_slice());

        json_array_builder.build()
    }
}

pub fn hash_map_to_vec(data: &HashMap<String, Vec<Arc<DbRow>>>) -> Vec<u8> {
    let mut json_array_builder = JsonArrayBuilder::new();

    for (_, db_rows) in data {
        for db_row in db_rows {
            json_array_builder.append_json_object(db_row.data.as_slice());
        }
    }

    json_array_builder.build()
}
