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
        let dt_as_string = &self.data[self.value_start + 1..self.value_end - 1];
        parse_date_time(dt_as_string)
    }
}

fn parse_date_time(time_as_string: &[u8]) -> Option<DateTimeAsMicroseconds> {
    if time_as_string.len() >= 26 {
        return parse_date_time_microseconds(time_as_string);
    }

    if time_as_string.len() >= 23 {
        return parse_date_time_milliseconds(time_as_string);
    }
    if time_as_string.len() == 19 {
        return parse_date_time_seconds(time_as_string);
    }
    None
}

const START_ZERO: u8 = '0' as u8;
//YYYY-MM-DDThh:mm:ss
fn parse_date_time_seconds(time_as_string: &[u8]) -> Option<DateTimeAsMicroseconds> {
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

    let date_time = NaiveDate::from_ymd(year, month, day).and_hms_micro(hour, min, sec, 0);

    let timestamp = date_time.timestamp_millis();

    Some(DateTimeAsMicroseconds::new(timestamp * 1000))
}
//YYYY-MM-DDThh:mm:ss.xxxxxxZ
fn parse_date_time_milliseconds(time_as_string: &[u8]) -> Option<DateTimeAsMicroseconds> {
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

    let microsec = (time_as_string[20] - START_ZERO) as u32 * 100
        + (time_as_string[21] - START_ZERO) as u32 * 10
        + (time_as_string[22] - START_ZERO) as u32;

    let date_time =
        NaiveDate::from_ymd(year, month, day).and_hms_micro(hour, min, sec, microsec * 1000);

    Some(DateTimeAsMicroseconds::new(
        date_time.timestamp_nanos() / 1000,
    ))
}

//YYYY-MM-DDThh:mm:ss.xxxxxxZ
fn parse_date_time_microseconds(time_as_string: &[u8]) -> Option<DateTimeAsMicroseconds> {
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

    let microsec = (time_as_string[20] - START_ZERO) as u32 * 100000
        + (time_as_string[21] - START_ZERO) as u32 * 10000
        + (time_as_string[22] - START_ZERO) as u32 * 1000
        + (time_as_string[23] - START_ZERO) as u32 * 100
        + (time_as_string[24] - START_ZERO) as u32 * 10
        + (time_as_string[25] - START_ZERO) as u32;

    let date_time = NaiveDate::from_ymd(year, month, day).and_hms_micro(hour, min, sec, microsec);

    Some(DateTimeAsMicroseconds::new(
        date_time.timestamp_nanos() / 1000,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_date_time_19() {
        let src_data = "2019-01-01T12:15:34".to_string();

        let i = parse_date_time(&src_data.into_bytes()).unwrap();
        println!("{}", i.to_rfc3339());
        assert_eq!("2019-01-01T12:15:34", &i.to_rfc3339()[..19]);
    }

    #[test]
    pub fn test_date_time_micros() {
        let src_data = "2022-03-17T13:28:29.6537478Z".to_string();

        let i = parse_date_time(&src_data.into_bytes()).unwrap();

        println!("{}", i.to_rfc3339());

        assert_eq!("2022-03-17T13:28:29.653747", &i.to_rfc3339()[..26]);
    }

    #[test]
    pub fn test_date_time_mill() {
        let src_data = "2022-03-17T13:28:29.653".to_string();

        let i = parse_date_time(&src_data.into_bytes()).unwrap();

        println!("{}", i.to_rfc3339());

        assert_eq!("2022-03-17T13:28:29.653", &i.to_rfc3339()[..23]);
    }
}
