use std::collections::HashMap;

use crate::db::FailOperationResult;

use super::{consts, JsonFirstLine};

pub struct DbEntity<'s> {
    pub partition_key: String,
    pub row_key: String,
    pub expires: Option<i64>,
    pub time_stamp: Option<i64>,
    pub raw: &'s [u8],
}

impl<'s> DbEntity<'s> {
    pub fn parse(raw: &'s [u8]) -> Result<Self, FailOperationResult> {
        let first_line = super::parser::parse_first_line(raw)?;

        let partition_key = get_json_field_as_string(&first_line, consts::PARTITION_KEY);

        if partition_key.is_none() {
            return Err(FailOperationResult::FieldPartitionKeyIsRequired);
        }

        let row_key = get_json_field_as_string(&first_line, consts::ROW_KEY);

        if row_key.is_none() {
            return Err(FailOperationResult::FieldRowKeyIsRequired);
        }

        let result = Self {
            raw,
            partition_key: partition_key.unwrap(),
            row_key: row_key.unwrap(),
            expires: get_json_field_as_timestamp(&first_line, consts::EXPIRES),
            time_stamp: get_json_field_as_timestamp(&first_line, consts::TIME_STAMP),
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
) -> Option<i64> {
    return first_lines.get(field_name)?.try_get_date();
}