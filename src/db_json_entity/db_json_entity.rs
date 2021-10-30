use std::{collections::BTreeMap, sync::Arc};

use rust_extensions::date_time::{AtomicDateTimeAsMicroseconds, DateTimeAsMicroseconds};

use crate::{
    db::DbRow,
    json::{array_parser::ArrayToJsonObjectsSplitter, JsonFirstLineParser},
};

use super::DbEntityParseFail;

pub struct DbJsonEntity<'s> {
    pub partition_key: &'s str,
    pub row_key: &'s str,
    pub expires: Option<DateTimeAsMicroseconds>,
    pub time_stamp: Option<DateTimeAsMicroseconds>,
    pub raw: &'s [u8],
}

impl<'s> DbJsonEntity<'s> {
    pub fn parse(raw: &'s [u8]) -> Result<Self, DbEntityParseFail> {
        let mut partition_key = None;
        let mut row_key = None;
        let mut expires = None;
        let mut time_stamp = None;

        for line in JsonFirstLineParser::new(raw) {
            let line = line?;

            let name = line.get_name()?;

            if name == super::consts::PARTITION_KEY {
                partition_key = Some(line.get_value()?)
            }

            if name == super::consts::ROW_KEY {
                row_key = Some(line.get_value()?)
            }

            if name == super::consts::EXPIRES {
                expires = line.get_value_as_date_time();
            }

            if name == super::consts::TIME_STAMP {
                time_stamp = line.get_value_as_date_time();
            }
        }

        if partition_key.is_none() {
            return Err(DbEntityParseFail::FieldPartitionKeyIsRequired);
        }

        if row_key.is_none() {
            return Err(DbEntityParseFail::FieldRowKeyIsRequired);
        }

        let result = Self {
            raw,
            partition_key: partition_key.unwrap(),
            row_key: row_key.unwrap(),
            expires,
            time_stamp,
        };

        return Ok(result);
    }

    pub fn to_db_row(&self) -> DbRow {
        let time_stamp = match self.time_stamp {
            Some(value) => value,
            None => DateTimeAsMicroseconds::now(),
        };

        return DbRow {
            row_key: self.row_key.to_string(),
            data: self.raw.to_vec(),
            expires: self.expires,
            time_stamp,
            last_read_access: AtomicDateTimeAsMicroseconds::now(),
        };
    }

    pub fn parse_as_btreemap(
        src: &'s [u8],
    ) -> Result<BTreeMap<String, Vec<Arc<DbRow>>>, DbEntityParseFail> {
        let mut result = BTreeMap::new();

        for json in src.split_array_json_to_objects() {
            let db_entity = DbJsonEntity::parse(json)?;
            let db_row = db_entity.to_db_row();
            if !result.contains_key(db_entity.partition_key) {
                result.insert(db_entity.partition_key.to_string(), Vec::new());

                result
                    .get_mut(db_entity.partition_key)
                    .unwrap()
                    .push(Arc::new(db_row))
            }
        }
        return Ok(result);
    }
}
