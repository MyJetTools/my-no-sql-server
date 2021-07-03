use crate::{json::db_entity::DbEntity, utils::date_time::MyDateTime};

pub struct DbRow {
    pub row_key: String,
    pub data: Vec<u8>,
    pub expires: Option<MyDateTime>,
    pub time_stamp: MyDateTime,
    pub last_access: MyDateTime,
}

impl DbRow {
    pub fn form_db_entity<'s>(src: &DbEntity<'s>) -> Self {
        let time_stamp = match src.time_stamp {
            Some(value) => value,
            None => MyDateTime::utc_now(),
        };

        return Self {
            row_key: src.row_key.to_string(),
            data: src.raw.to_vec(),
            expires: src.expires,
            time_stamp,
            last_access: MyDateTime::utc_now(),
        };
    }

    pub fn update_last_access(&self, now: MyDateTime) {
        self.last_access.update_unsafe(now);
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
