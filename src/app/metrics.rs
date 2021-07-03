use std::collections::HashMap;

use prometheus::{Encoder, Gauge, Opts, Registry, TextEncoder};
use tokio::sync::RwLock;

pub struct PrometheusMetrics {
    registry: Registry,

    table_partitions: RwLock<HashMap<String, Gauge>>,
}

impl PrometheusMetrics {
    pub fn new() -> Self {
        Self {
            registry: Registry::new(),
            table_partitions: RwLock::new(HashMap::new()),
        }
    }

    async fn update_table_partitions_if_exists_amount(
        &self,
        table_name: &str,
        value: usize,
    ) -> bool {
        let read_access = self.table_partitions.read().await;

        if read_access.contains_key(table_name) {
            let gauge = read_access.get(table_name).unwrap();
            gauge.set(value as f64);
            return true;
        }

        return false;
    }

    pub async fn update_table_partitions_amount(&self, table_name: &str, value: usize) {
        if self
            .update_table_partitions_if_exists_amount(table_name, value)
            .await
        {
            return;
        }

        let mut write_access = self.table_partitions.write().await;

        if !write_access.contains_key(table_name) {
            let gauge_opts = Opts::new(
                format!("{}_table_partitions_amount", table_name),
                format!("{} partitions amount", table_name),
            );
            let gauge = Gauge::with_opts(gauge_opts).unwrap();
            self.registry.register(Box::new(gauge.clone())).unwrap();
            write_access.insert(table_name.to_string(), gauge);
        }

        let gauge = write_access.get(table_name).unwrap();
        gauge.set(value as f64);
    }

    pub fn build(&self) -> String {
        let mut buffer = vec![];
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        encoder.encode(&metric_families, &mut buffer).unwrap();

        return String::from_utf8(buffer).unwrap();
    }
}
