use super::super::super::consts;
use crate::json::JsonParseError;

pub struct LookingForNextKeyStartState {
    pub pos: usize,
}

impl LookingForNextKeyStartState {
    pub fn new(pos: usize) -> Self {
        Self { pos }
    }
    pub fn read_next(&self, raw: &[u8]) -> Result<Option<usize>, JsonParseError> {
        let mut pos = self.pos;

        while pos < raw.len() {
            let b = raw[pos];

            if super::utils::is_space(b) || b == consts::COMMA {
                pos += 1;
                continue;
            }

            if b == consts::DOUBLE_QUOTE {
                return Ok(Some(pos));
            }

            if b == consts::CLOSE_BRACKET {
                return Ok(None);
            }

            return Err(JsonParseError::new(format!(
                "We are expecting '\"' or '{}' sign. But we have {} at pos {}",
                consts::CLOSE_BRACKET,
                b as char,
                pos
            )));
        }

        return Err(JsonParseError::new(format!(
            "We are expecting '\"' or '{}' sign. But we have not found it at all",
            consts::CLOSE_BRACKET
        )));
    }
}
