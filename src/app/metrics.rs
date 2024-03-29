use my_tcp_sockets::ThreadsStatistics;
use prometheus::{Encoder, IntGauge, IntGaugeVec, Opts, Registry, TextEncoder};

use crate::operations::DbTableMetrics;

pub trait UpdatePendingToSyncModel {
    fn get_name(&self) -> &str;
    fn get_pending_to_sync(&self) -> usize;
}

pub struct PrometheusMetrics {
    registry: Registry,
    partitions_amount: IntGaugeVec,
    table_size: IntGaugeVec,
    persist_amount: IntGaugeVec,
    tcp_connections: IntGaugeVec,
    tcp_connections_changes: IntGaugeVec,
    http_connections_count: IntGauge,
    persist_delay_in_seconds: IntGaugeVec,
    pending_to_sync: IntGaugeVec,
}

const TABLE_NAME: &str = "table_name";
const TCP_METRIC: &str = "tcp_metric";

impl PrometheusMetrics {
    pub fn new() -> Self {
        let registry = Registry::new();
        let partitions_amount = create_partitions_amount_gauge();
        let table_size = create_table_size_gauge();
        let persist_amount = create_persist_amount_gauge();
        let tcp_connections = create_tcp_connections();
        let tcp_connections_changes = create_tcp_connections_changes();
        let fatal_errors_count = create_fatal_errors_count();

        let pending_to_sync = create_pending_to_sync();

        let persist_delay_in_seconds = create_persist_delay_in_seconds();

        let http_connections_count = create_http_connections_count();

        registry
            .register(Box::new(http_connections_count.clone()))
            .unwrap();

        registry
            .register(Box::new(partitions_amount.clone()))
            .unwrap();

        registry.register(Box::new(table_size.clone())).unwrap();
        registry.register(Box::new(persist_amount.clone())).unwrap();
        registry
            .register(Box::new(fatal_errors_count.clone()))
            .unwrap();

        registry
            .register(Box::new(tcp_connections.clone()))
            .unwrap();

        registry
            .register(Box::new(tcp_connections_changes.clone()))
            .unwrap();

        registry
            .register(Box::new(persist_delay_in_seconds.clone()))
            .unwrap();

        registry
            .register(Box::new(pending_to_sync.clone()))
            .unwrap();

        return Self {
            registry,
            partitions_amount,
            table_size,
            persist_amount,
            tcp_connections,
            tcp_connections_changes,
            persist_delay_in_seconds,
            pending_to_sync,
            http_connections_count,
        };
    }

    pub fn update_table_metrics(
        &self,
        table_name: &str,
        table_metrics: &DbTableMetrics,
        http_connections_count: i64,
    ) {
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

        self.http_connections_count.set(http_connections_count);
    }

    pub fn update_persist_delay(&self, table_name: &str, persist_delay: i64) {
        self.persist_delay_in_seconds
            .with_label_values(&[table_name])
            .set(persist_delay);
    }

    pub fn get_http_connections_amount(&self) -> i64 {
        self.http_connections_count.get()
    }

    pub fn update_pending_to_sync<TUpdatePendingToSyncModel: UpdatePendingToSyncModel>(
        &self,
        data_reader_connection: &TUpdatePendingToSyncModel,
    ) {
        let name = data_reader_connection.get_name();

        let pending_to_sync = data_reader_connection.get_pending_to_sync();

        self.pending_to_sync
            .with_label_values(&[&name])
            .set(pending_to_sync as i64);
    }

    pub fn remove_pending_to_sync<TUpdatePendingToSyncModel: UpdatePendingToSyncModel>(
        &self,
        data_reader_connection: &TUpdatePendingToSyncModel,
    ) {
        let name = data_reader_connection.get_name();

        let result = self.pending_to_sync.remove_label_values(&[&name]);

        if let Err(err) = result {
            println!(
                "Can not remove pending to sync metric for data reader {}: {:?}",
                name, err
            );
        }
    }
    pub fn mark_new_tcp_connection(&self) {
        self.tcp_connections.with_label_values(&["count"]).inc();
        self.tcp_connections_changes
            .with_label_values(&["connected"])
            .inc();
    }

    pub fn update_tcp_threads(&self, threads_statistics: &ThreadsStatistics) {
        self.tcp_connections
            .with_label_values(&["ping_threads"])
            .set(threads_statistics.ping_threads.get());

        self.tcp_connections
            .with_label_values(&["read_threads"])
            .set(threads_statistics.read_threads.get());

        self.tcp_connections
            .with_label_values(&["write_threads"])
            .set(threads_statistics.write_threads.get());

        self.tcp_connections
            .with_label_values(&["connection_objects"])
            .set(threads_statistics.connections_objects.get());
    }

    pub fn mark_new_tcp_disconnection(&self) {
        self.tcp_connections.with_label_values(&["count"]).dec();
        self.tcp_connections_changes
            .with_label_values(&["disconnected"])
            .inc();
    }

    pub fn build(&self) -> String {
        let mut buffer = vec![];
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        encoder.encode(&metric_families, &mut buffer).unwrap();

        return String::from_utf8(buffer).unwrap();
    }
}

fn create_partitions_amount_gauge() -> IntGaugeVec {
    let gauge_opts = Opts::new(
        format!("table_partitions_amount"),
        format!("table partitions amount"),
    );

    let labels = &[TABLE_NAME];
    IntGaugeVec::new(gauge_opts, labels).unwrap()
}

fn create_table_size_gauge() -> IntGaugeVec {
    let gauge_opts = Opts::new(format!("table_size"), format!("table size"));

    let labels = &[TABLE_NAME];
    IntGaugeVec::new(gauge_opts, labels).unwrap()
}

fn create_persist_amount_gauge() -> IntGaugeVec {
    let gauge_opts = Opts::new(format!("persist_amount"), format!("persist amount"));

    let labels = &[TABLE_NAME];
    IntGaugeVec::new(gauge_opts, labels).unwrap()
}

fn create_pending_to_sync() -> IntGaugeVec {
    let gauge_opts = Opts::new(
        format!("pending_to_send"),
        format!("pending bytes to send to reader"),
    );

    let labels = &[TABLE_NAME];
    IntGaugeVec::new(gauge_opts, labels).unwrap()
}

fn create_fatal_errors_count() -> IntGauge {
    IntGauge::new("fatal_errors_count", "Fatal errors count").unwrap()
}

fn create_http_connections_count() -> IntGauge {
    IntGauge::new("http_connections_count", "Http connections count").unwrap()
}

fn create_persist_delay_in_seconds() -> IntGaugeVec {
    let gauge_opts = Opts::new(
        format!("persist_delay_sec"),
        format!("Current delay of persistence operation in seconds"),
    );

    let labels = &[TABLE_NAME];
    IntGaugeVec::new(gauge_opts, labels).unwrap()
}

fn create_tcp_connections_changes() -> IntGaugeVec {
    let gauge_opts = Opts::new(format!("tcp_changes_count"), format!("Tcp Changes Count"));

    let labels = &[TCP_METRIC];
    IntGaugeVec::new(gauge_opts, labels).unwrap()
}

fn create_tcp_connections() -> IntGaugeVec {
    let gauge_opts = Opts::new(format!("tcp_connections"), format!("Tcp Connections"));
    let labels = &[TCP_METRIC];
    IntGaugeVec::new(gauge_opts, labels).unwrap()
}
