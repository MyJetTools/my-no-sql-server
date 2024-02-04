use std::sync::Arc;

use my_no_sql_sdk::core::rust_extensions::MyTimerTick;

use crate::app::AppContext;

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
        crate::operations::persist(&self.app).await;
    }
}
