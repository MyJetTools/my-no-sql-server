use std::time::Duration;

use rust_extensions::date_time::DateTimeAsMicroseconds;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct RequestMetric {
    pub moment: DateTimeAsMicroseconds,
    pub name: String,
    pub duration: Duration,
    pub status_code: u16,
    pub result_size: usize,
}

pub struct RequestMetrics {
    data: Mutex<Vec<RequestMetric>>,
}

impl RequestMetrics {
    pub fn new() -> Self {
        Self {
            data: Mutex::new(Vec::new()),
        }
    }

    pub async fn add_metric(
        &self,
        name: String,
        duration: Duration,
        status_code: u16,
        result_size: usize,
    ) {
        let metric = RequestMetric {
            moment: DateTimeAsMicroseconds::now(),
            name: name,
            duration,
            status_code,
            result_size,
        };

        let mut data = self.data.lock().await;
        data.push(metric);

        while data.len() > 100 {
            data.remove(0);
        }
    }

    pub async fn get_metrics(&self) -> Vec<RequestMetric> {
        let data = self.data.lock().await;
        data.clone()
    }
}
