use super::{
    super::consts,
    states::{LookingForJsonValueStartState, ReadingObjectValueState},
    JsonFirstLine,
};
use crate::json::json_first_line_reader::states::{
    LookingForNextKeyStartState, ReadingNonStringValueState,
};

use super::{
    read_mode::ReadMode,
    states::{LookingForJsonTokenState, ReadingStringState},
};
use crate::json::JsonParseError;

pub struct JsonFirstLineReader<'s> {
    raw: &'s [u8],
    read_mode: ReadMode,
}

impl<'s> JsonFirstLineReader<'s> {
    pub fn new(raw: &'s [u8]) -> Self {
        Self {
            raw,
            read_mode: ReadMode::LookingForOpenJson(LookingForJsonTokenState::new(
                0,
                consts::OPEN_BRACKET,
            )),
        }
    }
}

impl<'s> Iterator for JsonFirstLineReader<'s> {
    type Item = Result<JsonFirstLine<'s>, JsonParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut key_start = None;
        let mut key_end = None;
        let mut value_start = None;
        loop {
            let result = self.read_mode.read_next(self.raw);

            if let Err(err) = result {
                return Some(Err(err));
            }

            match result.unwrap() {
                super::ReadResult::OpenJsonFound(pos) => {
                    self.read_mode = ReadMode::LookingForJsonKeyStart(
                        LookingForJsonTokenState::new(pos + 1, consts::DOUBLE_QUOTE),
                    );
                }
                super::ReadResult::KeyStartFound(pos) => {
                    key_start = Some(pos);
                    self.read_mode = ReadMode::ReadingKey(ReadingStringState::new(pos));
                }
                super::ReadResult::KeyEndFound(pos) => {
                    key_end = Some(pos);
                    self.read_mode = ReadMode::LookingForKeyValueSeparator(
                        LookingForJsonTokenState::new(pos + 1, consts::DOUBLE_COLUMN),
                    );
                }
                super::ReadResult::KeyValueSeparatorFound(pos) => {
                    self.read_mode =
                        ReadMode::LookingForValueStart(LookingForJsonValueStartState::new(pos + 1));
                }
                super::ReadResult::FoundStringValueStart(pos) => {
                    value_start = Some(pos);
                    self.read_mode = ReadMode::ReadingStringValue(ReadingStringState::new(pos));
                }
                super::ReadResult::FoundNonStringValueStart(pos) => {
                    value_start = Some(pos);
                    self.read_mode =
                        ReadMode::ReadingNonStringValue(ReadingNonStringValueState::new(pos));
                }
                super::ReadResult::FoundObjectOrArrayValueStart(pos) => {
                    value_start = Some(pos);
                    self.read_mode =
                        ReadMode::ReadingObjectValue(ReadingObjectValueState::new(pos));
                }
                super::ReadResult::ValueEndFound(pos) => {
                    self.read_mode =
                        ReadMode::LookingForNextKeyStart(LookingForNextKeyStartState::new(pos + 1));

                    return Some(Ok(JsonFirstLine {
                        data: self.raw,
                        name_start: key_start.unwrap(),
                        name_end: key_end.unwrap() + 1,
                        value_start: value_start.unwrap(),
                        value_end: pos + 1,
                    }));
                }
                super::ReadResult::EndOfJson => {
                    return None;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    pub fn test_simple_parse() {
        let src_data = "{\"name1\":\"123\", \"name2\":true,       \"name3\":null, \"name4\":0.12}";

        let mut parser = JsonFirstLineReader::new(src_data.as_bytes());

        let item = parser.next().unwrap().unwrap();

        assert_eq!("\"name1\"", item.get_raw_name().unwrap());
        assert_eq!("\"123\"", item.get_raw_value().unwrap());

        let item = parser.next().unwrap().unwrap();

        assert_eq!("\"name2\"", item.get_raw_name().unwrap());
        assert_eq!("true", item.get_raw_value().unwrap());

        let item = parser.next().unwrap().unwrap();

        assert_eq!("\"name3\"", item.get_raw_name().unwrap());
        assert_eq!("null", item.get_raw_value().unwrap());

        let item = parser.next().unwrap().unwrap();

        assert_eq!("\"name4\"", item.get_raw_name().unwrap());
        assert_eq!("0.12", item.get_raw_value().unwrap());

        let item = parser.next();

        assert_eq!(true, item.is_none());
    }
}
