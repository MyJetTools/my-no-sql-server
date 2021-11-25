use std::{collections::VecDeque, sync::Arc};

use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::db::DbRow;

pub struct Multipart {
    pub created: DateTimeAsMicroseconds,
    pub id: i64,
    pub items: VecDeque<Arc<DbRow>>,
}

impl Multipart {
    pub fn new(now: DateTimeAsMicroseconds, items: VecDeque<Arc<DbRow>>) -> Self {
        Self {
            created: now,
            id: now.unix_microseconds,
            items,
        }
    }
}
