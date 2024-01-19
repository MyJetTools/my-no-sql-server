use std::sync::Arc;

use my_no_sql_sdk::core::{
    db::DbRow,
    db_json_entity::{DbJsonEntity, JsonTimeStamp},
};

use super::DbOperationError;

pub fn parse_grouped_by_partition_key(
    as_bytes: &[u8],
    inject_time_stamp: &JsonTimeStamp,
) -> Result<Vec<(String, Vec<Arc<DbRow>>)>, DbOperationError> {
    match DbJsonEntity::parse_grouped_by_partition_key(as_bytes, inject_time_stamp) {
        Ok(result) => Ok(result),
        Err(err) => {
            let result = DbOperationError::DbEntityParseFail(err);
            Err(result)
        }
    }
}
