use crate::{date_time::MyDateTime, db::FailOperationResult, json::JsonFirstLineParser};

pub struct DbEntity<'s> {
    pub partition_key: String,
    pub row_key: String,
    pub expires: Option<MyDateTime>,
    pub time_stamp: Option<MyDateTime>,
    pub raw: &'s [u8],
}

impl<'s> DbEntity<'s> {
    pub fn parse(raw: &'s [u8]) -> Result<Self, FailOperationResult> {
        let mut partition_key = None;
        let mut row_key = None;
        let mut expires = None;
        let mut time_stamp = None;

        for line in JsonFirstLineParser::new(raw) {
            let line = line?;

            let name = line.get_name()?;

            if name == super::consts::PARTITION_KEY {
                partition_key = Some(line.get_value()?.to_string())
            }

            if name == super::consts::ROW_KEY {
                row_key = Some(line.get_value()?.to_string())
            }

            if name == super::consts::EXPIRES {
                expires = line.get_value_as_date_time();
            }

            if name == super::consts::TIME_STAMP {
                time_stamp = line.get_value_as_date_time();
            }
        }

        if partition_key.is_none() {
            return Err(FailOperationResult::FieldPartitionKeyIsRequired);
        }

        if row_key.is_none() {
            return Err(FailOperationResult::FieldRowKeyIsRequired);
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
}
