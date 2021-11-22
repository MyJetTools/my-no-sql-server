use crate::json::JsonParseError;

use super::states::{
    LookingForJsonTokenState, LookingForJsonValueStartState, LookingForNextKeyStartState,
    ReadingNonStringValueState, ReadingObjectValueState, ReadingStringState,
};

use super::super::consts;

pub enum ReadMode {
    LookingForOpenJson(LookingForJsonTokenState),
    LookingForJsonKeyStart(LookingForJsonTokenState),
    ReadingKey(ReadingStringState),
    LookingForKeyValueSeparator(LookingForJsonTokenState),
    LookingForValueStart(LookingForJsonValueStartState),
    ReadingStringValue(ReadingStringState),
    ReadingNonStringValue(ReadingNonStringValueState),
    ReadingObjectValue(ReadingObjectValueState),
    LookingForNextKeyStart(LookingForNextKeyStartState),
}

pub enum ReadResult {
    OpenJsonFound(usize),
    KeyStartFound(usize),
    KeyEndFound(usize),
    KeyValueSeparatorFound(usize),
    FoundStringValueStart(usize),
    FoundNonStringValueStart(usize),
    FoundObjectOrArrayValueStart(usize),
    ValueEndFound(usize),
    EndOfJson,
}

impl ReadMode {
    pub fn read_next(&self, raw: &[u8]) -> Result<ReadResult, JsonParseError> {
        match self {
            ReadMode::LookingForOpenJson(state) => {
                let pos = state.read_next(raw)?;
                Ok(ReadResult::OpenJsonFound(pos))
            }
            ReadMode::LookingForJsonKeyStart(state) => {
                let pos = state.read_next(raw)?;
                Ok(ReadResult::KeyStartFound(pos))
            }
            ReadMode::ReadingKey(state) => {
                let pos = state.read_next(raw)?;
                Ok(ReadResult::KeyEndFound(pos))
            }
            ReadMode::LookingForKeyValueSeparator(state) => {
                let pos = state.read_next(raw)?;
                Ok(ReadResult::KeyValueSeparatorFound(pos))
            }
            ReadMode::LookingForValueStart(state) => {
                let pos = state.read_next(raw)?;

                let b = raw[pos];

                match b {
                    consts::DOUBLE_QUOTE => Ok(ReadResult::FoundStringValueStart(pos)),
                    consts::OPEN_BRACKET => Ok(ReadResult::FoundObjectOrArrayValueStart(pos)),
                    consts::OPEN_ARRAY => Ok(ReadResult::FoundObjectOrArrayValueStart(pos)),
                    _ => Ok(ReadResult::FoundNonStringValueStart(pos)),
                }
            }
            ReadMode::ReadingStringValue(state) => {
                let pos = state.read_next(raw)?;
                return Ok(ReadResult::ValueEndFound(pos));
            }
            ReadMode::ReadingNonStringValue(state) => {
                let pos = state.read_next(raw)?;
                return Ok(ReadResult::ValueEndFound(pos));
            }
            ReadMode::ReadingObjectValue(state) => {
                let pos = state.read_next(raw)?;
                return Ok(ReadResult::ValueEndFound(pos));
            }
            ReadMode::LookingForNextKeyStart(state) => {
                let pos = state.read_next(raw)?;

                match pos {
                    Some(pos) => Ok(ReadResult::KeyStartFound(pos)),
                    None => Ok(ReadResult::EndOfJson),
                }
            }
        }
    }
}
