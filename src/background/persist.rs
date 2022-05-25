use std::sync::Arc;

use rust_extensions::MyTimerTick;

use crate::{
    app::{logs::SystemProcess, AppContext},
    db::DbTable,
    persist_operations::data_to_persist::PersistResult,
};

pub struct PersistTimer {
    app: Arc<AppContext>,
}

impl PersistTimer {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl MyTimerTick for PersistTimer {
    async fn tick(&self) {
        let is_shutting_down = self.app.states.is_shutting_down();

        let tables = self.app.db.get_tables().await;

        for db_table in tables {
            if let Some(persist_result) = db_table.get_what_to_persist(is_shutting_down).await {
                let result =
                    tokio::spawn(persist(self.app.clone(), db_table.clone(), persist_result)).await;

                if let Err(err) = result {
                    self.app.logs.add_error(
                        Some(db_table.name.to_string()),
                        SystemProcess::PersistOperation,
                        "PersistTimer".to_string(),
                        "Panic during persist operation".to_string(),
                        Some(format!("{:?}", err)),
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
