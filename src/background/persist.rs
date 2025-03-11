use std::sync::Arc;

use my_no_sql_sdk::core::rust_extensions::MyTimerTick;
use my_no_sql_sdk::server::rust_extensions::date_time::DateTimeAsMicroseconds;

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
        let started = DateTimeAsMicroseconds::now();
        while (DateTimeAsMicroseconds::now() - started).get_full_seconds() < 30 {
            crate::operations::persist(&self.app).await;
        }
    }
}
