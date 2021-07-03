use std::sync::atomic::AtomicI64;

use super::MyDateTime;

pub struct AtomicDateTime {
    miliseconds: AtomicI64,
}

impl AtomicDateTime {
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
}
