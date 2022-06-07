#[cfg(test)]
use std::str::Utf8Error;

use super::{super::consts, JsonValue};
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

    pub fn get_value(&self) -> Result<JsonValue<'t>, JsonParseError> {
        let value = &self.data[self.value_start..self.value_end];

        if is_null(value) {
            return Ok(JsonValue::Null);
        }

        if let Some(value) = is_bool(value) {
            return Ok(JsonValue::Boolean(value));
        }

        if is_number(value) {
            return Ok(JsonValue::Number(convert_to_utf8(value)?));
        }

        if value[0] == consts::OPEN_ARRAY {
            return Ok(JsonValue::Array(value));
        }

        if value[0] == consts::OPEN_BRACKET {
            return Ok(JsonValue::Object(value));
        }

        return Ok(JsonValue::String(convert_to_utf8(value)?));
    }

    /*
    pub fn get_value_as_date_time(&self) -> Option<DateTimeAsMicroseconds> {

    }
    */
}

fn convert_to_utf8(src: &[u8]) -> Result<&str, JsonParseError> {
    match std::str::from_utf8(src) {
        Ok(str) => Ok(str),
        Err(err) => Err(JsonParseError::new(format!(
            "Can convert value to utf8 string. Err {}",
            err
        ))),
    }
}

const NULL_LC: [u8; 4] = [b'n', b'u', b'l', b'l'];
const NULL_UC: [u8; 4] = [b'N', b'U', b'L', b'L'];

fn is_null(src: &[u8]) -> bool {
    if is_that_value(&NULL_LC, &NULL_UC, src) {
        return true;
    }

    return false;
}

fn is_number(src: &[u8]) -> bool {
    return src[0] >= '0' as u8 && src[9] <= '9' as u8;
}

const TRUE_LC: [u8; 4] = [b't', b'r', b'u', b'e'];
const TRUE_UC: [u8; 4] = [b'T', b'R', b'U', b'E'];

const FALSE_LC: [u8; 5] = [b'f', b'a', b'l', b's', b'e'];
const FALSE_UC: [u8; 5] = [b'F', b'A', b'L', b'S', b'E'];

fn is_bool(src: &[u8]) -> Option<bool> {
    if is_that_value(&TRUE_LC, &TRUE_UC, src) {
        return Some(true);
    }

    if is_that_value(&FALSE_LC, &FALSE_UC, src) {
        return Some(true);
    }

    None
}

fn is_that_value(src_lc: &[u8], src_uc: &[u8], dest: &[u8]) -> bool {
    if src_lc.len() != dest.len() {
        return false;
    }

    let mut pos = 0;

    for b in dest {
        if *b != src_lc[pos] && *b != src_uc[pos] {
            return false;
        }

        pos += 1;
    }

    return true;
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_null_str() {
        assert_eq!(false, is_null("15".as_bytes()));

        assert_eq!(true, is_null("null".as_bytes()));
        assert_eq!(true, is_null("Null".as_bytes()));
    }
}
