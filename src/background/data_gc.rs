use std::{sync::Arc, time::Duration};

use rust_extensions::{date_time::DateTimeAsMicroseconds, MyTimerTick};

use crate::{app::AppContext, db_sync::EventSource};

pub struct DataGcTimer {
    app: Arc<AppContext>,
}

impl DataGcTimer {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl MyTimerTick for DataGcTimer {
    async fn tick(&self) {
        let multipart_timeout = Duration::from_secs(60);

        let now = DateTimeAsMicroseconds::now();

        self.app.multipart_list.gc(now, multipart_timeout).await;

        let tables = self.app.db.get_tables().await;

        for db_table in tables {
            let max_partitions_amount = db_table.attributes.get_max_partitions_amount();

            if max_partitions_amount.is_none() {
                continue;
            }

            let max_partitions_amount = max_partitions_amount.unwrap();

            let mut persist_moment = now.clone();
            persist_moment.add_seconds(1);

            crate::db_operations::gc::keep_max_partitions_amount::execute(
                self.app.as_ref(),
                db_table,
                max_partitions_amount,
                EventSource::as_gc(),
                persist_moment,
            )
            .await;
        }
    }
}
