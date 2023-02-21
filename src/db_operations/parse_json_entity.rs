use std::{collections::BTreeMap, sync::Arc};

use my_no_sql_core::{
    db::DbRow,
    db_json_entity::{DbJsonEntity, JsonTimeStamp},
};

use super::DbOperationError;

pub fn as_single_entity(as_bytes: &[u8]) -> Result<DbJsonEntity, DbOperationError> {
    match DbJsonEntity::parse(as_bytes) {
        Ok(result) => Ok(result),
        Err(err) => {
            let result = DbOperationError::DbEntityParseFail(err);
            Err(result)
        }
    }
}

pub fn parse_as_btree_map(
    as_bytes: &[u8],
    inject_time_stamp: &JsonTimeStamp,
) -> Result<BTreeMap<String, Vec<Arc<DbRow>>>, DbOperationError> {
    match DbJsonEntity::parse_as_btreemap(as_bytes, inject_time_stamp) {
        Ok(result) => Ok(result),
        Err(err) => {
            let result = DbOperationError::DbEntityParseFail(err);
            Err(result)
        }
    }
}
