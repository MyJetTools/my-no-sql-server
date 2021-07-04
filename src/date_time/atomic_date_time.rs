use std::{sync::atomic::AtomicI64, time::Duration};

use super::MyDateTime;

pub struct AtomicDateTime {
    miliseconds: AtomicI64,
}

impl AtomicDateTime {
    pub fn from_date_time(dt: MyDateTime) -> Self {
        Self {
            miliseconds: AtomicI64::new(dt.miliseconds),
        }
    }
    pub fn utc_now() -> Self {
        let miliseconds = super::utils::get_utc_now();
        Self {
            miliseconds: AtomicI64::new(miliseconds),
        }
    }

    pub fn update(&self, value: MyDateTime) {
        self.miliseconds
            .store(value.miliseconds, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn update_value(&self, value: i64) {
        self.miliseconds
            .store(value, std::sync::atomic::Ordering::SeqCst);
    }
    pub fn get(&self) -> i64 {
        return self.miliseconds.load(std::sync::atomic::Ordering::SeqCst);
    }

    pub fn clone(&self) -> Self {
        Self {
            miliseconds: AtomicI64::new(self.get()),
        }
    }

    pub fn duration_to(&self, now: MyDateTime) -> Option<Duration> {
        let miliseconds = self.get();
        if now.miliseconds > miliseconds {
            let milis = now.miliseconds - miliseconds;
            return Some(Duration::from_millis(milis as u64));
        }

        return None;
    }
}
