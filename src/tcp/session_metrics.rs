use rust_extensions::date_time::DateTimeAsMicroseconds;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct SessionMetricsData {
    pub session_id: u64,
    pub ip: String,
    pub read_amount: usize,
    pub last_incoming_moment: DateTimeAsMicroseconds,
    pub connected: DateTimeAsMicroseconds,
    pub name: Option<String>,
}

impl SessionMetricsData {
    pub fn new(session_id: u64, ip: String) -> Self {
        Self {
            read_amount: 0,
            last_incoming_moment: DateTimeAsMicroseconds::now(),
            connected: DateTimeAsMicroseconds::now(),
            session_id,
            ip,
            name: None,
        }
    }
}

pub struct SessionMetrics {
    data: Mutex<SessionMetricsData>,
}

impl SessionMetrics {
    pub fn new(id: u64, ip: String) -> Self {
        Self {
            data: Mutex::new(SessionMetricsData::new(id, ip)),
        }
    }

    pub async fn increase_read_size(&self, read_size: usize, moment: DateTimeAsMicroseconds) {
        let mut write_access = self.data.lock().await;

        write_access.read_amount += read_size;
        write_access.last_incoming_moment = moment;
    }

    pub async fn get_metrics(&self) -> SessionMetricsData {
        let read_access = self.data.lock().await;
        return read_access.clone();
    }

    pub async fn update_name(&self, name: String) {
        let mut write_access = self.data.lock().await;
        write_access.name = Some(name);
    }

    pub async fn get_incoming_traffic_moment(&self) -> DateTimeAsMicroseconds {
        let read_access = self.data.lock().await;
        read_access.last_incoming_moment
    }
}
