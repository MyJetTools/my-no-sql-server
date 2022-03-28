use std::sync::Arc;

use rust_extensions::{date_time::DateTimeAsMicroseconds, MyTimerTick};

use crate::{app::AppContext, db::DbTable, db_sync::EventSource, utils::LazyVec};

pub struct GcDbRows {
    app: Arc<AppContext>,
}

impl GcDbRows {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl MyTimerTick for GcDbRows {
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
    let now = DateTimeAsMicroseconds::now();
    let tables_with_something_to_gc = get_tables_which_has_something_to_gc(app.as_ref(), now).await;

    if let Some(tables) = tables_with_something_to_gc {
        expire_partitions_and_rows(app.as_ref(), tables, now).await;
    }
}

pub async fn get_tables_which_has_something_to_gc(
    app: &AppContext,
    now: DateTimeAsMicroseconds,
) -> Option<Vec<Arc<DbTable>>> {
    let mut result = LazyVec::new();

    let tables = app.db.get_tables().await;

    for table in tables {
        let table_read_access = table.data.read().await;

        if let Some(max_partitions_amount) = table.attributes.get_max_partitions_amount() {
            if table_read_access.partitions.len() > max_partitions_amount {
                result.push(table.clone());
                continue;
            }
        }

        for db_partition in table_read_access.partitions.values() {
            if db_partition.rows.has_rows_to_expire(now) {
                result.push(table.clone());
            }
        }
    }

    result.get_result()
}

async fn expire_partitions_and_rows(
    app: &AppContext,
    tables: Vec<Arc<DbTable>>,
    now: DateTimeAsMicroseconds,
) {
    for db_table in tables {
        let max_partitions_amount = db_table.attributes.get_max_partitions_amount();

        if let Some(max_partitions_amount) = max_partitions_amount {
            crate::db_operations::gc::keep_max_partitions_amount_and_expire_db_rows(
                app,
                db_table,
                max_partitions_amount,
                EventSource::as_gc(),
                crate::app::DEFAULT_PERSIST_PERIOD.get_sync_moment(),
                now,
            )
            .await;
        }
    }
}
