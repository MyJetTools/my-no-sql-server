use rust_extensions::date_time::DateTimeAsMicroseconds;

#[derive(Clone)]
pub struct CommonStateData {
    pub table_name: String,
    pub sync_moment: Option<DateTimeAsMicroseconds>,
    pub sync_table_attrs: bool,
}

impl CommonStateData {
    pub fn update_sync_moment_if_needed(&mut self, new_moment: DateTimeAsMicroseconds) {
        match &mut self.sync_moment {
            Some(sync_moment) => {
                if new_moment.unix_microseconds < sync_moment.unix_microseconds {
                    sync_moment.unix_microseconds = new_moment.unix_microseconds
                }
            }
            None => self.sync_moment = Some(new_moment),
        }
    }
}
