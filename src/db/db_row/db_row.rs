use std::sync::atomic::AtomicI64;

use rust_extensions::date_time::{AtomicDateTimeAsMicroseconds, DateTimeAsMicroseconds};

pub struct DbRow {
    pub partition_key: String,
    pub row_key: String,
    pub data: Vec<u8>,
    expires: AtomicI64,
    pub time_stamp: DateTimeAsMicroseconds,
    pub last_read_access: AtomicDateTimeAsMicroseconds,
}

impl DbRow {
    pub fn new(
        partition_key: String,
        row_key: String,
        data: Vec<u8>,
        expires: Option<DateTimeAsMicroseconds>,
        time_stamp: DateTimeAsMicroseconds,
    ) -> Self {
        let last_read_access = AtomicDateTimeAsMicroseconds::new(time_stamp.unix_microseconds);

        Self {
            partition_key,
            row_key,
            data,
            expires: expires_to_atomic(expires),
            time_stamp,
            last_read_access,
        }
    }

    pub fn update_last_access(&self, now: DateTimeAsMicroseconds) {
        self.last_read_access.update(now);
    }

    pub fn get_expires(&self) -> Option<DateTimeAsMicroseconds> {
        let result = self.expires.load(std::sync::atomic::Ordering::SeqCst);

        if result == NULL_EXPIRES {
            return None;
        }

        return Some(DateTimeAsMicroseconds::new(result));
    }
}

const NULL_EXPIRES: i64 = 0;

fn expires_to_atomic(expires: Option<DateTimeAsMicroseconds>) -> AtomicI64 {
    if let Some(expires) = expires {
        if expires.unix_microseconds == NULL_EXPIRES {
            return AtomicI64::new(NULL_EXPIRES + 1);
        }

        return AtomicI64::new(expires.unix_microseconds);
    }

    return AtomicI64::new(NULL_EXPIRES);
}
