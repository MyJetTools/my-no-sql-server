use std::collections::HashMap;

use tokio::sync::Mutex;

use super::http_metrics_by_url::HttpMetricsByUrl;

pub struct HttpMetrics {
    pub metrics: Mutex<HashMap<String, HttpMetricsByUrl>>,
}

impl HttpMetrics {
    pub fn new() -> Self {
        Self {
            metrics: Mutex::new(HashMap::new()),
        }
    }

    pub async fn add(&self, url: String, http_status_code: u16, microseconds: i64) {
        let mut metrics = self.metrics.lock().await;

        if let Some(metrics_by_url) = metrics.get_mut(&url) {
            metrics_by_url.add(http_status_code as u8, microseconds);
            return;
        }

        let mut new_metrics_by_url = HttpMetricsByUrl::new();
        new_metrics_by_url.add(http_status_code as u8, microseconds);
        metrics.insert(url, new_metrics_by_url);
    }
    pub async fn get_snapshot(&self) -> HashMap<String, HttpMetricsByUrl> {
        let metrics = self.metrics.lock().await;

        metrics.clone()
    }
}
