use crate::json::json_utils::FoundResult;
use crate::json::JsonParseError;
pub struct LookingForJsonTokenState {
    pub pos: usize,
    pub token: u8,
}

impl LookingForJsonTokenState {
    pub fn new(pos: usize, token: u8) -> Self {
        Self { pos, token }
    }
    pub fn read_next(&self, raw: &[u8]) -> Result<usize, JsonParseError> {
        let result = super::super::super::json_utils::next_token_must_be(raw, self.pos, self.token);
        match result {
            FoundResult::Ok(pos) => Ok(pos),
            FoundResult::EndOfJson => Err(JsonParseError::new(format!(
                "We started looking for a token {} at pos {} and did not found",
                self.token, self.pos
            ))),
            FoundResult::InvalidTokenFound { found_token, pos } => {
                Err(JsonParseError::new(format!(
                "We started looking for a token {} at pos {} but we found a token '{}' at pos {}",
                self.token as char, self.pos, found_token as char, pos
            )))
            }
        }
    }
}
