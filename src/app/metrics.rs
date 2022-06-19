use prometheus::{Encoder, IntGauge, IntGaugeVec, Opts, Registry, TextEncoder};

use crate::db::DbTableMetrics;

pub struct PrometheusMetrics {
    registry: Registry,
    partitions_amount: IntGaugeVec,
    table_size: IntGaugeVec,
    persist_amount: IntGaugeVec,
    tcp_connections_count: IntGauge,
    tcp_connections_changes: IntGaugeVec,
    fatal_errors_count: IntGauge,
    persist_delay_in_seconds: IntGaugeVec,
}

const TABLE_NAME: &str = "table_name";
const TCP_METRIC: &str = "tcp_metric";

impl PrometheusMetrics {
    pub fn new() -> Self {
        let registry = Registry::new();
        let partitions_amount = create_partititions_amount_gauge();
        let table_size = create_table_size_gauge();
        let persist_amount = create_persist_amount_gauge();
        let tcp_connections_count = create_tcp_connections_count();
        let tcp_connections_changes = create_tcp_connections_changes();
        let fatal_errors_count = create_fatal_errors_count();

        let persist_delay_in_seconds = create_persist_delay_in_seconds();

        registry
            .register(Box::new(partitions_amount.clone()))
            .unwrap();

        registry.register(Box::new(table_size.clone())).unwrap();
        registry.register(Box::new(persist_amount.clone())).unwrap();
        registry
            .register(Box::new(fatal_errors_count.clone()))
            .unwrap();

        registry
            .register(Box::new(tcp_connections_count.clone()))
            .unwrap();

        registry
            .register(Box::new(tcp_connections_changes.clone()))
            .unwrap();

        registry
            .register(Box::new(persist_delay_in_seconds.clone()))
            .unwrap();

        return Self {
            registry,
            partitions_amount,
            table_size,
            persist_amount,
            tcp_connections_count,
            tcp_connections_changes,
            fatal_errors_count,
            persist_delay_in_seconds,
        };
    }

    pub fn update_table_metrics(&self, table_name: &str, table_metrics: &DbTableMetrics) {
        let partitions_amount_value = table_metrics.partitions_amount as i64;
        self.partitions_amount
            .with_label_values(&[table_name])
            .set(partitions_amount_value);

        let table_size_value = table_metrics.table_size as i64;
        self.table_size
            .with_label_values(&[table_name])
            .set(table_size_value);

        let persist_amount_value = table_metrics.persist_amount as i64;
        self.persist_amount
            .with_label_values(&[table_name])
            .set(persist_amount_value);
    }

    pub fn update_persist_delay(&self, table_name: &str, persist_delay: i64) {
        self.persist_delay_in_seconds
            .with_label_values(&[table_name])
            .set(persist_delay);
    }

    pub fn mark_new_tcp_connection(&self) {
        self.tcp_connections_count.inc();
        self.tcp_connections_changes
            .with_label_values(&["connected"])
            .inc();
    }

    pub fn mark_new_tcp_disconnection(&self) {
        self.tcp_connections_count.dec();
        self.tcp_connections_changes
            .with_label_values(&["disconnected"])
            .inc();
    }

    pub fn update_fatal_errors_count(&self, value: i64) {
        self.fatal_errors_count.set(value);
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

fn create_persist_amount_gauge() -> IntGaugeVec {
    let gauge_opts = Opts::new(format!("persist_amount"), format!("persist amount"));

    let lables = &[TABLE_NAME];
    IntGaugeVec::new(gauge_opts, lables).unwrap()
}

fn create_fatal_errors_count() -> IntGauge {
    IntGauge::new("fatal_erros_count", "Fatal errors count").unwrap()
}
fn create_tcp_connections_count() -> IntGauge {
    IntGauge::new("tcp_connections_count", "TCP Connections count").unwrap()
}

fn create_persist_delay_in_seconds() -> IntGaugeVec {
    let gauge_opts = Opts::new(
        format!("persist_delay_sec"),
        format!("Current delay of persistence operation in seconds"),
    );

    let lables = &[TABLE_NAME];
    IntGaugeVec::new(gauge_opts, lables).unwrap()
}

fn create_tcp_connections_changes() -> IntGaugeVec {
    let gauge_opts = Opts::new(format!("tcp_changes_count"), format!("Tcp Changes Count"));

    let lables = &[TCP_METRIC];
    IntGaugeVec::new(gauge_opts, lables).unwrap()
}
