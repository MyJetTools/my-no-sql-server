use std::sync::Arc;

use rust_extensions::MyTimerTick;

use crate::{app::AppContext, persist_operations::PersistResult};

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
                match persist_result {
                    PersistResult::PersistAttrs => {
                        crate::operations::persist::io_with_cache::save_table_attributes(
                            self.app.as_ref(),
                            db_table.as_ref(),
                        )
                        .await;
                    }
                    PersistResult::PersistTable => {
                        crate::operations::persist::persist_table::execute(
                            self.app.as_ref(),
                            db_table.as_ref(),
                        )
                        .await;
                    }
                    PersistResult::PersistPartition(partition_key) => {
                        crate::operations::persist::persist_partition::execute(
                            self.app.as_ref(),
                            db_table.as_ref(),
                            partition_key.as_str(),
                        )
                        .await;
                    }
                }
            }
        }
    }
}
