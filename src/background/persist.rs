use std::sync::Arc;

use rust_extensions::MyTimerTick;

use crate::{
    app::{logs::SystemProcess, AppContext},
    db::DbTable,
    persist_operations::data_to_persist::PersistResult,
};

pub enum TimerType {
    Dedicated(Arc<DbTable>),
    Common,
}

pub struct PersistTimer {
    app: Arc<AppContext>,
    timer_type: TimerType,
}

impl PersistTimer {
    pub fn new(app: Arc<AppContext>, timer_type: TimerType) -> Self {
        Self { app, timer_type }
    }
}

#[async_trait::async_trait]
impl MyTimerTick for PersistTimer {
    async fn tick(&self) {
        let is_shutting_down = self.app.states.is_shutting_down();

        let tables = match &self.timer_type {
            TimerType::Dedicated(db_table) => vec![db_table.clone()],
            TimerType::Common => self.app.db.get_tables_with_common_persist_thread().await,
        };

        for db_table in tables {
            if let Some(persist_result) = db_table.get_what_to_persist(is_shutting_down).await {
                let result =
                    tokio::spawn(persist(self.app.clone(), db_table.clone(), persist_result)).await;

                if let Err(err) = result {
                    self.app.logs.add_fatal_error(
                        Some(db_table.name.to_string()),
                        SystemProcess::PersistOperation,
                        "PersistTimer".to_string(),
                        format!("Can not persist messages {:?}", err),
                    )
                }
            }
        }
    }
}

async fn persist(app: Arc<AppContext>, db_table: Arc<DbTable>, persist_result: PersistResult) {
    match persist_result {
        PersistResult::PersistAttrs => {
            let attrs = db_table.attributes.get_snapshot();
            crate::persist_operations::sync::save_table_attributes(
                app.as_ref(),
                db_table.name.as_str(),
                &attrs,
            )
            .await;
            db_table.update_last_persist_time().await;
        }
        PersistResult::PersistTable => {
            crate::persist_operations::sync::save_table(app.as_ref(), db_table.as_ref()).await;

            db_table.update_last_persist_time().await;
        }
        PersistResult::PersistPartition(partition_key) => {
            crate::persist_operations::sync::save_partition(
                app.as_ref(),
                db_table.as_ref(),
                partition_key.as_str(),
            )
            .await;
            db_table.update_last_persist_time().await;
        }
    }
}
