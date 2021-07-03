use crate::{json::db_entity::DbEntity, utils::date_time};

pub struct DbRow {
    pub row_key: String,
    pub data: Vec<u8>,
    pub expires: Option<i64>,
    pub time_stamp: i64,
    pub last_access: i64,
}

impl DbRow {
    pub fn form_db_entity<'s>(src: &DbEntity<'s>) -> Self {
        let time_stamp = match src.time_stamp {
            Some(value) => value,
            None => date_time::get_utc_now(),
        };

        return Self {
            row_key: src.row_key.to_string(),
            data: src.raw.to_vec(),
            expires: src.expires,
            time_stamp,
            last_access: date_time::get_utc_now(),
        };
    }

    pub fn update_last_access(&self, now: i64) {
        unsafe {
            let const_ptr = self.last_access as *const i64;
            let mut_ptr = const_ptr as *mut i64;
            *mut_ptr = now;
        }
    }
}

impl Clone for DbRow {
    fn clone(&self) -> Self {
        Self {
            row_key: self.row_key.clone(),
            data: self.data.clone(),
            expires: self.expires.clone(),
            time_stamp: self.time_stamp,
            last_access: self.last_access,
        }
    }
}
