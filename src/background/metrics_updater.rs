use std::sync::Arc;

use rust_extensions::MyTimerTick;

use crate::app::AppContext;

pub struct MetricsUpdater {
    app: Arc<AppContext>,
}

impl MetricsUpdater {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl MyTimerTick for MetricsUpdater {
    async fn tick(&self) {
        let tables = self.app.db.get_tables().await;

        let mut persist_amount = 0;

        for db_table in tables {
            let table_metrics = db_table.get_metrics().await;

            persist_amount += table_metrics.persist_amount;

            let persist_delay = if table_metrics.last_persist_time.unix_microseconds
                < table_metrics.last_update_time.unix_microseconds
            {
                let duration = table_metrics
                    .last_update_time
                    .duration_since(table_metrics.last_persist_time);

                duration.as_secs() as i64
            } else {
                0
            };

            self.app
                .metrics
                .update_table_metrics(db_table.name.as_str(), &table_metrics);

            self.app
                .metrics
                .update_persist_delay(db_table.name.as_str(), persist_delay);
        }

        self.app.update_persist_amount(persist_amount);

        let fatal_errors_count = self.app.logs.get_fatal_errors_amount();

        self.app
            .metrics
            .update_fatal_errors_count(fatal_errors_count);

        for reader in self.app.data_readers.get_all().await {
            self.app
                .metrics
                .update_pending_to_sync(&reader.connection)
                .await;
        }
    }
}
