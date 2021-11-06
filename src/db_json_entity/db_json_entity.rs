use std::{collections::BTreeMap, sync::Arc};

use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    db::DbRow,
    json::{array_parser::ArrayToJsonObjectsSplitter, JsonFirstLineParser},
};

use super::{date_time_injector::TimeStampValuePosition, utils::JsonTimeStamp, DbEntityParseFail};

pub struct DbJsonEntity<'s> {
    pub partition_key: &'s str,
    pub row_key: &'s str,
    pub expires: Option<DateTimeAsMicroseconds>,
    pub time_stamp: Option<DateTimeAsMicroseconds>,
    timestamp_value_position: Option<TimeStampValuePosition>,
    raw: &'s [u8],
}

impl<'s> DbJsonEntity<'s> {
    pub fn parse(raw: &'s [u8]) -> Result<Self, DbEntityParseFail> {
        let mut partition_key = None;
        let mut row_key = None;
        let mut expires = None;
        let mut time_stamp = None;
        let mut timestamp_value_position = None;

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
                timestamp_value_position = Some(TimeStampValuePosition {
                    start: line.value_start,
                    end: line.value_end,
                });

                time_stamp = line.get_value_as_date_time()
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
            timestamp_value_position,
        };

        return Ok(result);
    }

    pub fn to_db_row(&self, time_stamp: DateTimeAsMicroseconds) -> DbRow {
        let data = compile_row_content(self.raw, &self.timestamp_value_position, time_stamp);

        return DbRow::new(
            self.partition_key.to_string(),
            self.row_key.to_string(),
            data,
            self.expires,
            time_stamp,
        );
    }

    pub fn restore_db_row(&self, time_stamp: DateTimeAsMicroseconds) -> DbRow {
        return DbRow::new(
            self.partition_key.to_string(),
            self.row_key.to_string(),
            self.raw.to_vec(),
            self.expires,
            time_stamp,
        );
    }

    pub fn parse_as_btreemap(
        src: &'s [u8],
        time_stamp: DateTimeAsMicroseconds,
    ) -> Result<BTreeMap<String, Vec<Arc<DbRow>>>, DbEntityParseFail> {
        let mut result = BTreeMap::new();

        for json in src.split_array_json_to_objects() {
            let db_entity = DbJsonEntity::parse(json)?;
            let db_row = db_entity.to_db_row(time_stamp);
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

fn compile_row_content(
    raw: &[u8],
    time_stamp_value_position: &Option<TimeStampValuePosition>,
    time_stamp: DateTimeAsMicroseconds,
) -> Vec<u8> {
    let json_time_stamp = JsonTimeStamp::new(time_stamp);

    if let Some(time_stamp_value_position) = time_stamp_value_position {
        return super::date_time_injector::replace_timestamp_value(
            raw,
            time_stamp_value_position,
            json_time_stamp,
        );
    } else {
        return super::date_time_injector::inject(raw, json_time_stamp);
    }
}
