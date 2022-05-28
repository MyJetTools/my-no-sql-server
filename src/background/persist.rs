use std::sync::Arc;

use rust_extensions::MyTimerTick;

use crate::{app::AppContext, operations::PersistType};

pub struct PersistTimer {
    app: Arc<AppContext>,
    persist_type: PersistType,
}

impl PersistTimer {
    pub fn new(app: Arc<AppContext>, persist_type: PersistType) -> Self {
        Self { app, persist_type }
    }
}

#[async_trait::async_trait]
impl MyTimerTick for PersistTimer {
    async fn tick(&self) {
        crate::operations::persist(&self.app, &self.persist_type).await;
    }
}
