use std::collections::HashMap;

use crate::{db::DbRow, json::array_parser::ArrayToJsonObjectsSplitter};

use super::{DbEntity, DbEntityParseFail};

pub fn parse_entities_to_hash_map_by_partition_key<'s>(
    src: &'s [u8],
) -> Result<HashMap<String, Vec<DbEntity>>, DbEntityParseFail> {
    let mut result = HashMap::new();

    for json in src.split_array_json_to_objects() {
        let db_entity = DbEntity::parse(json)?;
        if !result.contains_key(db_entity.partition_key.as_str()) {
            result.insert(db_entity.partition_key.to_string(), Vec::new());

            result
                .get_mut(db_entity.partition_key.as_str())
                .unwrap()
                .push(db_entity)
        }
    }
    return Ok(result);
}

pub fn parse_db_rows_to_hash_map_by_partition_key(
    src: &[u8],
) -> Result<HashMap<String, Vec<DbRow>>, DbEntityParseFail> {
    let mut result = HashMap::new();

    for json in src.split_array_json_to_objects() {
        let db_entity = DbEntity::parse(json)?;

        let db_row = DbRow::form_db_entity(&db_entity);
        if !result.contains_key(db_entity.partition_key.as_str()) {
            result.insert(db_entity.partition_key.to_string(), Vec::new());

            result
                .get_mut(db_entity.partition_key.as_str())
                .unwrap()
                .push(db_row)
        }
    }
    return Ok(result);
}
