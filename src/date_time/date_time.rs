use std::time::{Duration, UNIX_EPOCH};

use chrono::{DateTime, Utc};

#[derive(Clone, Copy, Debug)]
pub struct MyDateTime {
    pub miliseconds: i64,
}

impl MyDateTime {
    pub fn new(miliseconds: i64) -> Self {
        Self { miliseconds }
    }
    pub fn utc_now() -> Self {
        let miliseconds = super::utils::get_utc_now();

        Self { miliseconds }
    }

    pub fn to_iso_string(&self) -> String {
        let d = UNIX_EPOCH + Duration::from_millis(self.miliseconds as u64);

        let datetime = DateTime::<Utc>::from(d);

        return format!("{:?}", datetime);
    }

    pub fn equals_to(&self, other_one: MyDateTime) -> bool {
        return self.miliseconds == other_one.miliseconds;
    }
}
