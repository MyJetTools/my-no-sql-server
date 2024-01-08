use std::sync::Arc;

use my_http_server::HttpConnectionsCounter;
use my_tcp_sockets::ThreadsStatistics;
use rust_extensions::MyTimerTick;

use crate::app::AppContext;

pub struct MetricsUpdater {
    app: Arc<AppContext>,
    http_connections_count: HttpConnectionsCounter,
    threads_statistics: Arc<ThreadsStatistics>,
}

impl MetricsUpdater {
    pub fn new(
        app: Arc<AppContext>,
        http_connections_count: HttpConnectionsCounter,
        threads_statistics: Arc<ThreadsStatistics>,
    ) -> Self {
        Self {
            app,
            http_connections_count,
            threads_statistics,
        }
    }
}

#[async_trait::async_trait]
impl MyTimerTick for MetricsUpdater {
    async fn tick(&self) {
        let tables = self.app.db.get_tables().await;

        let mut persist_amount = 0;

        self.app
            .metrics
            .update_tcp_threads(&self.threads_statistics);

        for db_table in tables {
            let table_metrics =
                crate::operations::get_table_metrics(self.app.as_ref(), db_table.as_ref()).await;

            persist_amount += table_metrics.persist_amount;

            let persist_delay = if let Some(last_persist_time) = table_metrics.last_persist_time {
                if last_persist_time.unix_microseconds
                    < table_metrics.last_update_time.unix_microseconds
                {
                    let duration = table_metrics
                        .last_update_time
                        .duration_since(last_persist_time)
                        .as_positive_or_zero();

                    duration.as_secs() as i64
                } else {
                    0
                }
            } else {
                0
            };

            let http_connections_amount = self.http_connections_count.get_connections_amount();

            self.app.metrics.update_table_metrics(
                db_table.name.as_str(),
                &table_metrics,
                http_connections_amount,
            );

            self.app
                .metrics
                .update_persist_delay(db_table.name.as_str(), persist_delay);
        }

        self.app.update_persist_amount(persist_amount);

        for reader in self.app.data_readers.get_all().await {
            self.app.metrics.update_pending_to_sync(&reader.connection);

            reader.connection.one_sec_tick().await;
        }
    }
}
