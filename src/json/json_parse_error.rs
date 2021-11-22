#[derive(Debug)]
pub struct JsonParseError {
    pub msg: String,
}

impl JsonParseError {
    pub fn new(msg: String) -> Self {
        Self { msg }
    }

    pub fn to_string(&self) -> String {
        return self.msg.to_string();
    }
}
