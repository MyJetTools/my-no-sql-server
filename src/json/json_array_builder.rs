pub struct JsonArrayBuilder {
    raw: Vec<u8>,
    first_line: bool,
}

impl JsonArrayBuilder {
    pub fn new() -> Self {
        Self {
            raw: vec![super::consts::OPEN_ARRAY],
            first_line: true,
        }
    }
    pub fn append_json_object(&mut self, raw_json_object: &[u8]) {
        if self.first_line {
            self.first_line = false;
        } else {
            self.raw.push(super::consts::COMMA)
        }

        self.raw.extend(raw_json_object);
    }

    pub fn build(mut self) -> Vec<u8> {
        self.raw.push(super::consts::CLOSE_ARRAY);
        return self.raw;
    }
}
