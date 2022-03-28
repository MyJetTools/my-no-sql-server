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

            self.app
                .metrics
                .update_table_metrics(db_table.name.as_str(), &table_metrics);
        }

        self.app.update_persist_amount(persist_amount);
    }
}
