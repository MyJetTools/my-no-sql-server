#[cfg(test)]
use std::str::Utf8Error;

use chrono::NaiveDate;
use rust_extensions::date_time::DateTimeAsMicroseconds;

use super::super::consts;
use crate::json::JsonParseError;

pub struct JsonFirstLine<'t> {
    pub name_start: usize,
    pub name_end: usize,
    pub value_start: usize,
    pub value_end: usize,
    pub data: &'t [u8],
}

impl<'t> JsonFirstLine<'t> {
    #[cfg(test)]
    pub fn get_raw_name(&self) -> Result<&'t str, Utf8Error> {
        let name = &self.data[self.name_start..self.name_end];
        return std::str::from_utf8(name);
    }

    pub fn get_name(&self) -> Result<&'t str, JsonParseError> {
        let name = &self.data[self.name_start + 1..self.name_end - 1];

        if name.len() == 0 {
            return Err(JsonParseError::new(format!(
                "Invalid name len: {}",
                name.len()
            )));
        }

        let result = std::str::from_utf8(name);
        match result {
            Ok(str) => Ok(str),
            Err(err) => Err(JsonParseError::new(format!(
                "Can convert name to utf8 string. Err {}",
                err
            ))),
        }
    }

    #[cfg(test)]
    pub fn get_raw_value(&self) -> Result<&'t str, Utf8Error> {
        let value = &self.data[self.value_start..self.value_end];
        return std::str::from_utf8(value);
    }

    pub fn get_value(&self) -> Result<&'t str, JsonParseError> {
        let mut value = &self.data[self.value_start..self.value_end];

        if value[0] == consts::DOUBLE_QUOTE {
            if value.len() < 2 {
                return Err(JsonParseError::new(format!(
                    "Value starts with '{}' but has a len: {}",
                    consts::DOUBLE_QUOTE,
                    value.len()
                )));
            }

            value = &value[1..value.len() - 1];
        }

        let result = std::str::from_utf8(value);
        match result {
            Ok(str) => Ok(str),
            Err(err) => Err(JsonParseError::new(format!(
                "Can convert value to utf8 string. Err {}",
                err
            ))),
        }
    }

    pub fn get_value_as_date_time(&self) -> Option<DateTimeAsMicroseconds> {
        parse_date_time(&self.data[self.value_start..self.value_end])
    }
}

fn parse_date_time(time_as_string: &[u8]) -> Option<DateTimeAsMicroseconds> {
    if time_as_string.len() == 19 {
        return parse_date_time_19(time_as_string);
    }

    None
}

const START_ZERO: u8 = '0' as u8;
//YYYY-MM-DDThh:mm:ss
fn parse_date_time_19(time_as_string: &[u8]) -> Option<DateTimeAsMicroseconds> {
    let year = (time_as_string[0] - START_ZERO) as i32 * 1000
        + (time_as_string[1] - START_ZERO) as i32 * 100
        + (time_as_string[2] - START_ZERO) as i32 * 10
        + (time_as_string[3] - START_ZERO) as i32;

    let month =
        (time_as_string[5] - START_ZERO) as u32 * 10 + (time_as_string[6] - START_ZERO) as u32;

    let day =
        (time_as_string[8] - START_ZERO) as u32 * 10 + (time_as_string[9] - START_ZERO) as u32;

    let hour =
        (time_as_string[11] - START_ZERO) as u32 * 10 + (time_as_string[12] - START_ZERO) as u32;

    let min =
        (time_as_string[14] - START_ZERO) as u32 * 10 + (time_as_string[15] - START_ZERO) as u32;

    let sec =
        (time_as_string[17] - START_ZERO) as u32 * 10 + (time_as_string[18] - START_ZERO) as u32;

    let date_time = NaiveDate::from_ymd(year, month, day).and_hms(hour, min, sec);

    Some(DateTimeAsMicroseconds::new(date_time.timestamp()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_date_time_19() {
        let src_data = "2019-01-01T12:00:00".to_string();

        let i = parse_date_time(&src_data.into_bytes()).unwrap();

        println!("{}", i.to_rfc3339());
    }
}
