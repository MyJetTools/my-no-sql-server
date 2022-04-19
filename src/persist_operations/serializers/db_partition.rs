use std::sync::Arc;

use crate::{
    db::DbPartition,
    db_json_entity::{DbJsonEntity, JsonTimeStamp},
    json::array_parser::ArrayToJsonObjectsSplitter,
};

pub fn deserialize(raw: &[u8]) -> Result<DbPartition, String> {
    let mut db_partition = DbPartition::new();

    for db_entity_json_result in raw.split_array_json_to_objects() {
        if let Err(err) = db_entity_json_result {
            return Err(format!("Can not split to array of json objects: {:?}", err));
        }

        let db_entity_json = db_entity_json_result.unwrap();
        let db_entity = DbJsonEntity::parse(db_entity_json);

        if let Err(err) = db_entity {
            return Err(format!("Can not parse DbJsonEntity: {:?}", err));
        }

        let db_entity = db_entity.unwrap();

        let db_row = if let Some(time_stamp) = db_entity.time_stamp {
            let time_stamp = JsonTimeStamp::parse_or_now(time_stamp);
            db_entity.restore_db_row(&time_stamp)
        } else {
            let time_stamp = JsonTimeStamp::now();
            db_entity.to_db_row(&time_stamp)
        };

        db_partition.insert_row(Arc::new(db_row), None);
    }
    Ok(db_partition)
}
