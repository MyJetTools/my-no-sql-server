use std::sync::Arc;

use rust_extensions::MyTimerTick;

use crate::{app::AppContext, db_sync::EventSource};

pub struct GcPartitions {
    app: Arc<AppContext>,
}

impl GcPartitions {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl MyTimerTick for GcPartitions {
    async fn tick(&self) {
        let result = tokio::spawn(gc_partitions_iteration(self.app.clone())).await;

        if let Err(err) = result {
            self.app.logs.add_fatal_error(
                crate::app::logs::SystemProcess::Timer,
                format!("gc_partitions"),
                format!("{}", err),
            );
        }
    }
}

async fn gc_partitions_iteration(app: Arc<AppContext>) {
    let tables_with_partition_limit = app.db.get_tables_which_partitions_restrictions().await;

    if let Some(tables_with_partition_limit) = tables_with_partition_limit {
        for db_table in tables_with_partition_limit {
            let max_partitions_amount = db_table.attributes.get_max_partitions_amount();

            if let Some(max_partitions_amount) = max_partitions_amount {
                crate::db_operations::gc::keep_max_partitions_amount::execute(
                    app.as_ref(),
                    db_table,
                    max_partitions_amount,
                    EventSource::as_gc(),
                    crate::app::DEFAULT_PERSIST_PERIOD.get_sync_moment(),
                )
                .await;
            }
        }
    }
}
