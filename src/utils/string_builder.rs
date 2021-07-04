use std::string::FromUtf8Error;

pub struct StringBuilder {
    buffer: Vec<u8>,
}

impl StringBuilder {
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    pub fn append_bytes(&mut self, bytes: &[u8]) {
        self.buffer.extend(bytes);
    }

    pub fn append(&mut self, s: &str) {
        self.buffer.extend(s.as_bytes());
    }

    pub fn append_line(&mut self, s: &str) {
        self.buffer.extend(s.as_bytes());
        self.buffer.push(b'\n');
    }

    pub fn append_u8(&mut self, c: u8) {
        self.buffer.push(c);
    }

    pub fn to_string_utf8(self) -> Result<String, FromUtf8Error> {
        return String::from_utf8(self.buffer);
    }
}
