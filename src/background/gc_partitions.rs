use std::{sync::Arc, time::Duration};

use crate::{app::AppContext, db_sync::EventSource};

pub async fn start(app: Arc<AppContext>) {
    let duration = Duration::from_secs(1);

    while !app.states.is_initialized() {
        tokio::time::sleep(duration).await;
    }

    while !app.states.is_shutting_down() {
        for _ in 0..60 {
            tokio::time::sleep(duration).await;

            if !app.states.is_initialized() {
                return;
            }
        }

        let result = tokio::spawn(gc_partitions_iteration(app.clone())).await;

        if let Err(err) = result {
            app.logs.add_fatal_error(
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
