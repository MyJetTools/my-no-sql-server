use crate::json::JsonParseError;

pub struct ReadingObjectValueState {
    pub pos: usize,
}

impl ReadingObjectValueState {
    pub fn new(pos: usize) -> Self {
        Self { pos }
    }

    pub fn read_next(&self, raw: &[u8]) -> Result<usize, JsonParseError> {
        super::super::super::json_utils::read_json_object(raw, self.pos)
    }
}
