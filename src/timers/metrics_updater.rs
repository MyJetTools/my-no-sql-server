use std::{sync::Arc, time::Duration};

use crate::app::AppServices;

pub async fn start(app: Arc<AppServices>) {
    let delay = Duration::from_secs(5);
    loop {
        tokio::time::sleep(delay).await;

        let tables = app.db.tables.read().await;

        for db_table in tables.values() {
            app.metrics
                .update_table_partitions_amount(
                    db_table.name.as_str(),
                    db_table.get_partitions_amount().await,
                )
                .await;
        }
    }
}
