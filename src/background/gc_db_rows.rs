use std::sync::Arc;

use rust_extensions::{date_time::DateTimeAsMicroseconds, MyTimerTick};

use crate::{app::AppContext, db_sync::EventSource};

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
        if !self.app.states.is_initialized() {
            return;
        }

        if self.app.states.is_shutting_down() {
            return;
        }

        gc_it(self.app.as_ref()).await;
    }
}

async fn gc_it(app: &AppContext) {
    use my_no_sql_server_core::logs::SystemProcess;
    let tables = app.db.get_tables().await;

    let now = DateTimeAsMicroseconds::now();

    for table in tables {
        let data_to_gc = {
            let table_data = table.data.read().await;
            table_data.get_data_to_gc(now)
        };

        if let Some(data_to_gc) = data_to_gc.get_data_to_gc() {
            let now = DateTimeAsMicroseconds::now();
            let mut persist_moment = now.clone();
            persist_moment.add_seconds(5);

            if data_to_gc.partitions.len() > 0 {
                println!("GcPartitions: {}", data_to_gc.partitions.len());
                if let Err(err) = crate::db_operations::write::delete_partitions(
                    app,
                    &table,
                    data_to_gc.partitions.into_iter().map(|x| x.0),
                    EventSource::GarbageCollector,
                    persist_moment,
                    now,
                )
                .await
                {
                    app.logs.add_error(
                        table.name.to_string().into(),
                        SystemProcess::Timer,
                        "GcPartitions".to_string().into(),
                        format!("{:?}", err),
                        None,
                    )
                }
            }

            if data_to_gc.db_rows.len() > 0 {
                println!("GcRows: {}", data_to_gc.db_rows.len());
                if let Err(err) = crate::db_operations::write::bulk_delete(
                    app,
                    &table,
                    data_to_gc.db_rows,
                    EventSource::GarbageCollector,
                    persist_moment,
                    now,
                )
                .await
                {
                    app.logs.add_error(
                        table.name.to_string().into(),
                        SystemProcess::Timer,
                        "GcRows".to_string().into(),
                        format!("{:?}", err),
                        None,
                    )
                }
            }
        }
    }
}
