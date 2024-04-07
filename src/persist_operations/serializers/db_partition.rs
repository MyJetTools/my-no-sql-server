use std::sync::Arc;

use my_no_sql_sdk::core::my_json::json_reader::array_iterator::JsonArrayIterator;
use my_no_sql_sdk::core::{db::DbPartition, db_json_entity::DbJsonEntity};
use my_no_sql_server_core::rust_extensions::array_of_bytes_iterator::SliceIterator;

pub fn deserialize(partition_key: &str, raw: &[u8]) -> Result<DbPartition, String> {
    let mut db_partition = DbPartition::new(partition_key.to_string());

    let mut json_array_iterator: JsonArrayIterator<SliceIterator> = raw.into();

    while let Some(db_entity_json_result) = json_array_iterator.get_next() {
        if let Err(err) = db_entity_json_result {
            return Err(format!("Can not split to array of json objects: {:?}", err));
        }

        let db_entity_json = db_entity_json_result.unwrap();

        match DbJsonEntity::restore_into_db_row(
            db_entity_json
                .unwrap_as_object(&json_array_iterator)
                .unwrap(),
        ) {
            Ok(db_row) => {
                db_partition.insert_row(Arc::new(db_row));
            }
            Err(err) => {
                println!("Skipping entity. Reason {:?}", err);
            }
        }
    }
    Ok(db_partition)
}
