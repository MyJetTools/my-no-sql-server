use rust_extensions::date_time::DateTimeAsMicroseconds;

pub struct JsonTimeStamp {
    src: String,
}

impl JsonTimeStamp {
    pub fn new(time_stamp: DateTimeAsMicroseconds) -> Self {
        Self {
            src: time_stamp.to_rfc3339(),
        }
    }

    pub fn as_str(&self) -> &str {
        return std::str::from_utf8(self.as_slice()).unwrap();
    }

    pub fn as_slice(&self) -> &[u8] {
        return &self.src.as_bytes()[..26];
    }
}
