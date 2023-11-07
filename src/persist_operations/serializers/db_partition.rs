use std::sync::Arc;

use my_json::json_reader::array_parser::ArrayToJsonObjectsSplitter;
use my_no_sql_sdk::core::{db::DbPartition, db_json_entity::DbJsonEntity};

pub fn deserialize(raw: &[u8]) -> Result<DbPartition, String> {
    let mut db_partition = DbPartition::new();

    for db_entity_json_result in raw.split_array_json_to_objects() {
        if let Err(err) = db_entity_json_result {
            return Err(format!("Can not split to array of json objects: {:?}", err));
        }

        let db_entity_json = db_entity_json_result.unwrap();

        match DbJsonEntity::parse(db_entity_json) {
            Ok(db_entity) => {
                let db_row = db_entity.restore_db_row();
                db_partition.insert_row(Arc::new(db_row));
            }
            Err(err) => {
                println!("Skipping entity. Reason {:?}", err);
            }
        }
    }
    Ok(db_partition)
}
