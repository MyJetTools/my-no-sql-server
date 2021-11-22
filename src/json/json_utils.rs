use super::{consts, JsonParseError};

pub enum FoundResult {
    Ok(usize),
    EndOfJson,
    InvalidTokenFound { found_token: u8, pos: usize },
}

pub fn read_json_object(raw: &[u8], start_pos: usize) -> Result<usize, JsonParseError> {
    let mut brackets = Vec::new();

    let b = raw[start_pos];

    if b == consts::OPEN_BRACKET || b == consts::OPEN_ARRAY {
        brackets.push(b);
    } else {
        panic!(
            "Bug... It has to be {} or {} symbol",
            consts::OPEN_BRACKET as char,
            consts::OPEN_ARRAY as char
        )
    }

    let mut pos = start_pos + 1;

    while pos < raw.len() {
        let b = raw[pos];

        match b {
            consts::DOUBLE_QUOTE => {
                let read_string_result = read_string(raw, pos);

                match read_string_result {
                    Some(end_string_pos) => {
                        pos = end_string_pos;
                    }
                    None => {
                        return Err(JsonParseError::new(format!(
                            "Error reading value as object. Start {}. Error pos {}",
                            start_pos, pos
                        )));
                    }
                }
            }
            consts::OPEN_ARRAY => {
                brackets.push(b);
            }
            consts::OPEN_BRACKET => {
                brackets.push(b);
            }

            consts::CLOSE_BRACKET => {
                let open_bracket = brackets[brackets.len() - 1];
                if open_bracket == consts::OPEN_BRACKET {
                    brackets.remove(brackets.len() - 1);
                    if brackets.len() == 0 {
                        return Ok(pos);
                    }
                } else {
                    return Err(JsonParseError::new(format!(
                        "Error reading value as object. Start {}. Error pos {}. Open bracket '{}' does not match close bracket '{}'",
                        start_pos, pos, open_bracket as u8,  b as u8
                    )));
                }
            }

            consts::CLOSE_ARRAY => {
                let open_bracket = brackets[brackets.len() - 1];
                if open_bracket == consts::OPEN_ARRAY {
                    brackets.remove(brackets.len() - 1);
                    if brackets.len() == 0 {
                        return Ok(pos);
                    }
                } else {
                    return Err(JsonParseError::new(format!(
                        "Error reading value as object. Start {}. Error pos {}. Open bracket '{}' does not match close bracket '{}'",
                        start_pos, pos, open_bracket as u8,  b as u8
                    )));
                }
            }

            _ => {}
        }
        pos += 1;
    }

    return Err(JsonParseError::new(format!(
        "Error reading value as object. Start {}. We reached the end of the payload",
        start_pos
    )));
}

pub fn read_string(raw: &[u8], start_pos: usize) -> Option<usize> {
    let mut esc_mode = false;
    let mut pos = start_pos + 1;
    while pos < raw.len() {
        let b = raw[pos];

        if esc_mode {
            esc_mode = false;
        } else {
            match b {
                consts::ESC_SYMBOL => {
                    esc_mode = true;
                }
                consts::DOUBLE_QUOTE => {
                    return Some(pos);
                }
                _ => {}
            }
        }

        pos += 1;
    }

    return None;
}

pub fn next_token_must_be(raw: &[u8], start_pos: usize, token: u8) -> FoundResult {
    let mut pos = start_pos;
    while pos < raw.len() {
        let b = raw[pos];
        if is_space(b) {
            pos += 1;
            continue;
        }

        if b == token {
            return FoundResult::Ok(pos);
        } else {
            return FoundResult::InvalidTokenFound {
                found_token: b,
                pos,
            };
        }
    }

    return FoundResult::EndOfJson;
}

pub fn is_space(c: u8) -> bool {
    c <= 32
}
