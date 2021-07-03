use std::{collections::HashMap, sync::Arc};

use crate::{
    db::{DbPartition, DbRow, DbTableData},
    json::JsonArrayBuilder,
};

pub trait DbEntityAsJsonArray {
    fn as_json_array(&self) -> Vec<u8>;
}

impl DbEntityAsJsonArray for DbTableData {
    fn as_json_array(&self) -> Vec<u8> {
        let mut json_array_builder = JsonArrayBuilder::new();

        for (_, db_partition) in &self.partitions {
            for (_, db_row) in &db_partition.rows {
                json_array_builder.append_json_object(db_row.data.as_slice());
            }
        }

        return json_array_builder.build();
    }
}

impl DbEntityAsJsonArray for DbPartition {
    fn as_json_array(&self) -> Vec<u8> {
        let mut json_array_builder = JsonArrayBuilder::new();
        for (_, db_rows) in &self.rows {
            json_array_builder.append_json_object(db_rows.data.as_slice());
        }

        return json_array_builder.build();
    }
}

impl DbEntityAsJsonArray for DbRow {
    fn as_json_array(&self) -> Vec<u8> {
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
