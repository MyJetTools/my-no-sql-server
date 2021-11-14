use std::{collections::BTreeMap, sync::Arc};

use crate::{
    db::{DbPartition, DbRow},
    json::JsonArrayBuilder,
};

pub trait DbEntityAsJsonArray {
    fn as_json_array(&self) -> Vec<u8>;
}

impl DbEntityAsJsonArray for [&DbRow] {
    fn as_json_array(&self) -> Vec<u8> {
        let mut json_array_builder = JsonArrayBuilder::new();

        for db_row in self {
            json_array_builder.append_json_object(&db_row.data)
        }

        return json_array_builder.build();
    }
}

impl DbEntityAsJsonArray for DbPartition {
    fn as_json_array(&self) -> Vec<u8> {
        let mut json_array_builder = JsonArrayBuilder::new();
        self.fill_with_json_data(&mut json_array_builder);
        return json_array_builder.build();
    }
}

impl DbEntityAsJsonArray for Vec<Arc<DbRow>> {
    fn as_json_array(&self) -> Vec<u8> {
        let mut json_array_builder = JsonArrayBuilder::new();

        for db_row in self {
            json_array_builder.append_json_object(&db_row.data);
        }

        json_array_builder.build()
    }
}

impl DbEntityAsJsonArray for BTreeMap<String, Vec<Arc<DbRow>>> {
    fn as_json_array(&self) -> Vec<u8> {
        let mut json_array_builder = JsonArrayBuilder::new();

        for db_partition in self.values() {
            for db_row in db_partition {
                json_array_builder.append_json_object(&db_row.data);
            }
        }

        json_array_builder.build()
    }
}

impl DbEntityAsJsonArray for DbRow {
    fn as_json_array(&self) -> Vec<u8> {
        let mut json_array_builder = JsonArrayBuilder::new();

        json_array_builder.append_json_object(self.data.as_slice());

        json_array_builder.build()
    }
}
