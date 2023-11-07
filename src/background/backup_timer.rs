use std::sync::Arc;

use rust_extensions::MyTimerTick;

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
    async fn tick(&self) {
        crate::operations::backup::save_backup(&self.app, false).await;
        crate::operations::backup::gc_backups(&self.app).await;
    }
}
