use std::sync::Arc;

use my_no_sql_sdk::core::rust_extensions::{MyTimerTick, RepeatTimerIteration};

use crate::app::AppContext;

pub struct BackupTimer {
    app: Arc<AppContext>,
}

impl BackupTimer {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl MyTimerTick for BackupTimer {
    async fn tick(&self) -> RepeatTimerIteration {
        // Failures are already reported by save_backup itself. They must not
        // short-circuit the tick: the collector below runs for every namespace,
        // including the ones which did get a snapshot.
        let _ = crate::operations::backup::save_backup(&self.app, false).await;
        crate::operations::backup::gc_backups(&self.app).await;

        RepeatTimerIteration::WithInterval
    }
}
