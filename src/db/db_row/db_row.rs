use rust_extensions::date_time::{AtomicDateTimeAsMicroseconds, DateTimeAsMicroseconds};

pub struct DbRow {
    pub row_key: String,
    pub data: Vec<u8>,
    pub expires: Option<DateTimeAsMicroseconds>,
    pub time_stamp: DateTimeAsMicroseconds,
    pub last_read_access: AtomicDateTimeAsMicroseconds,
}

impl DbRow {
    pub fn update_last_access(&self, now: DateTimeAsMicroseconds) {
        self.last_read_access.update(now);
    }
}
