use crate::{app::AppContext, db_sync::SyncEvent};
use my_no_sql_sdk::core::rust_extensions::events_loop::EventsLoopTick;
use std::sync::Arc;

pub struct SyncEventLoop {
    app: Arc<AppContext>,
}

impl SyncEventLoop {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl EventsLoopTick<SyncEvent> for SyncEventLoop {
    async fn started(&self) {}
    async fn tick(&self, model: SyncEvent) {
        crate::operations::sync::sync(&self.app, &model).await;
    }

    async fn finished(&self) {}
}
