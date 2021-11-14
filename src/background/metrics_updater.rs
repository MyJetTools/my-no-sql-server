use std::{sync::Arc, time::Duration};

use crate::app::{logs::SystemProcess, AppContext};

pub async fn start(app: Arc<AppContext>) {
    app.logs
        .add_info(
            None,
            SystemProcess::System,
            "Timer metrics updaters readers gc".to_string(),
            "Started".to_string(),
        )
        .await;
    let delay = Duration::from_secs(5);
    loop {
        tokio::time::sleep(delay).await;

        let tables = app.db.get_tables().await;

        for db_table in tables {
            let table_metrics = db_table.get_metrics().await;

            app.metrics
                .update_table_metrics(db_table.name.as_str(), &table_metrics);
        }
    }
}
