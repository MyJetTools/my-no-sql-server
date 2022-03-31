#[cfg(test)]
use std::str::Utf8Error;

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
        crate::json::date_time::parse(dt_as_string)
    }
}
