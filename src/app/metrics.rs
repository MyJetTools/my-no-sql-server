use prometheus::{Encoder, IntGaugeVec, Opts, Registry, TextEncoder};

use crate::db::DbTableMetrics;

pub struct PrometheusMetrics {
    registry: Registry,
    partitions_amount: IntGaugeVec,
    table_size: IntGaugeVec,
}

const TABLE_NAME: &str = "table_name";
impl PrometheusMetrics {
    pub fn new() -> Self {
        let registry = Registry::new();
        let partitions_amount = create_partititions_amount_gauge();
        let table_size = create_table_size_gauge();

        registry
            .register(Box::new(partitions_amount.clone()))
            .unwrap();

        registry.register(Box::new(table_size.clone())).unwrap();

        return Self {
            registry,
            partitions_amount,
            table_size,
        };
    }

    pub fn update_table_metrics(&self, table_name: &str, table_metrics: &DbTableMetrics) {
        let value = table_metrics.partitions_amount as i64;
        self.partitions_amount
            .with_label_values(&[table_name])
            .set(value);

        let value = table_metrics.table_size as i64;
        self.table_size.with_label_values(&[table_name]).set(value);
    }

    pub fn build(&self) -> String {
        let mut buffer = vec![];
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        encoder.encode(&metric_families, &mut buffer).unwrap();

        return String::from_utf8(buffer).unwrap();
    }
}

fn create_partititions_amount_gauge() -> IntGaugeVec {
    let gauge_opts = Opts::new(
        format!("table_partitions_amount"),
        format!("table partitions amount"),
    );

    let lables = &[TABLE_NAME];
    IntGaugeVec::new(gauge_opts, lables).unwrap()
}

fn create_table_size_gauge() -> IntGaugeVec {
    let gauge_opts = Opts::new(format!("table_size"), format!("table size"));

    let lables = &[TABLE_NAME];
    IntGaugeVec::new(gauge_opts, lables).unwrap()
}
