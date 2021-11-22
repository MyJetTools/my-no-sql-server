use super::super::super::consts;
use crate::json::JsonParseError;

pub struct ReadingNonStringValueState {
    pub pos: usize,
}

impl ReadingNonStringValueState {
    pub fn new(pos: usize) -> Self {
        Self { pos }
    }

    pub fn read_next(&self, raw: &[u8]) -> Result<usize, JsonParseError> {
        let mut pos = self.pos + 1;

        while pos < raw.len() {
            let b = raw[pos];

            if is_non_string_value_char(b) {
                pos += 1;
                continue;
            }

            if b == consts::COMMA || super::utils::is_space(b) || b == consts::CLOSE_BRACKET {
                return Ok(pos - 1);
            }

            return Err(JsonParseError::new(format!(
                "Error reading non string value. Start {}, current pos {}",
                self.pos, pos
            )));
        }

        return Err(JsonParseError::new(format!(
            "Error reading non string value. Start {}. We reached the end of the payload",
            self.pos
        )));
    }
}

fn is_non_string_value_char(b: u8) -> bool {
    return super::utils::is_number(b) || super::utils::is_latin_letter(b) || b == '.' as u8;
}
