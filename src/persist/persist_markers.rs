use std::time::Duration;

use rust_extensions::date_time::DateTimeAsMicroseconds;

use super::partition_persist_marker::PartitionPersistMarker;
const DURATION_MONITORING_DATA_SIZE: usize = 120;

pub struct TablePersistData {
    pub data_to_persist: PartitionPersistMarker,
    pub persist_duration: Vec<usize>,
    pub last_persist_time: Option<DateTimeAsMicroseconds>,
}

impl TablePersistData {
    pub fn new() -> Self {
        Self {
            data_to_persist: PartitionPersistMarker::new(),
            persist_duration: Vec::with_capacity(DURATION_MONITORING_DATA_SIZE),
            last_persist_time: None,
        }
    }

    pub fn add_persist_duration(&mut self, dur: Duration) {
        while self.persist_duration.len() == DURATION_MONITORING_DATA_SIZE {
            self.persist_duration.remove(0);
        }

        self.persist_duration.push(dur.as_micros() as usize);

        self.last_persist_time = DateTimeAsMicroseconds::now().into();
    }
}
