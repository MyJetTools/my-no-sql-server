use std::collections::HashMap;

use crate::{date_time::MyDateTime, db::FailOperationResult, json::JsonFirstLine};

pub struct DbEntity<'s> {
    pub partition_key: String,
    pub row_key: String,
    pub expires: Option<MyDateTime>,
    pub time_stamp: Option<MyDateTime>,
    pub raw: &'s [u8],
}

impl<'s> DbEntity<'s> {
    pub fn parse(raw: &'s [u8]) -> Result<Self, FailOperationResult> {
        let first_line = crate::json::parser::parse_first_line(raw)?;

        let partition_key = get_json_field_as_string(&first_line, super::consts::PARTITION_KEY);

        if partition_key.is_none() {
            return Err(FailOperationResult::FieldPartitionKeyIsRequired);
        }

        let row_key = get_json_field_as_string(&first_line, super::consts::ROW_KEY);

        if row_key.is_none() {
            return Err(FailOperationResult::FieldRowKeyIsRequired);
        }

        let result = Self {
            raw,
            partition_key: partition_key.unwrap(),
            row_key: row_key.unwrap(),
            expires: get_json_field_as_timestamp(&first_line, super::consts::EXPIRES),
            time_stamp: get_json_field_as_timestamp(&first_line, super::consts::TIME_STAMP),
        };

        return Ok(result);
    }
}

fn get_json_field_as_string<'s>(
    first_lines: &'s HashMap<&'s str, JsonFirstLine<'s>>,
    field_name: &str,
) -> Option<String> {
    let result = first_lines.get(field_name)?;
    let value = result.get_value();

    return match value {
        Ok(result) => Some(result.to_string()),
        Err(_) => None,
    };
}

fn get_json_field_as_timestamp<'s>(
    first_lines: &'s HashMap<&'s str, JsonFirstLine>,
    field_name: &'s str,
) -> Option<MyDateTime> {
    return first_lines.get(field_name)?.try_get_date();
}
