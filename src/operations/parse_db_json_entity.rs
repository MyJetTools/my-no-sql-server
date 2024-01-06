use my_no_sql_sdk::core::{
    db::DbRow,
    db_json_entity::{DbJsonEntity, DbJsonEntityWithContent, JsonTimeStamp},
};

use crate::db_operations::DbOperationError;

pub fn parse_db_json_entity(src: &[u8], now: &JsonTimeStamp) -> Result<DbRow, DbOperationError> {
    match DbJsonEntity::parse_into_db_row(src, now) {
        Ok(result) => Ok(result),
        Err(err) => {
            let result = DbOperationError::DbEntityParseFail(err);
            Err(result)
        }
    }
}

pub fn parse_db_json_entity_to_validate<'s>(
    src: &'s [u8],
    now: &'s JsonTimeStamp,
) -> Result<DbJsonEntityWithContent<'s>, DbOperationError> {
    match DbJsonEntity::parse(src, now) {
        Ok(result) => Ok(result),
        Err(err) => {
            let result = DbOperationError::DbEntityParseFail(err);
            Err(result)
        }
    }
}
