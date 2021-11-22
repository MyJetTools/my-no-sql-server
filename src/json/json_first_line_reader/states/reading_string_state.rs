use crate::json::JsonParseError;

pub struct ReadingStringState {
    pub pos: usize,
}

impl ReadingStringState {
    pub fn new(pos: usize) -> Self {
        Self { pos }
    }

    pub fn read_next(&self, raw: &[u8]) -> Result<usize, JsonParseError> {
        let result = super::super::super::json_utils::read_string(raw, self.pos);

        match result {
            Some(pos) => Ok(pos),
            None => Err(JsonParseError::new(format!(
                "Can not find end of the string starting from the pos {}",
                self.pos
            ))),
        }
    }
}
