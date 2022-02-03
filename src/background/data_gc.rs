use std::{sync::Arc, time::Duration};

use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    app::{logs::SystemProcess, AppContext},
    db_sync::EventSource,
};

pub async fn start(app: Arc<AppContext>) {
    let delay = Duration::from_secs(10);
    app.logs
        .add_info(
            None,
            SystemProcess::System,
            "Timer dead data readers gc".to_string(),
            "Started".to_string(),
        )
        .await;

    let multipart_timeout = Duration::from_secs(60);

    loop {
        tokio::time::sleep(delay).await;

        let now = DateTimeAsMicroseconds::now();

        app.multipart_list.gc(now, multipart_timeout).await;

        let tables = app.db.get_tables().await;

        for db_table in tables {
            let max_partitions_amount = db_table.attributes.get_max_partitions_amount();

            if max_partitions_amount.is_none() {
                continue;
            }

            let max_partitions_amount = max_partitions_amount.unwrap();

            let mut persist_moment = now.clone();
            persist_moment.add_seconds(1);

            crate::db_operations::gc::keep_max_partitions_amount::execute(
                app.as_ref(),
                db_table,
                max_partitions_amount,
                EventSource::as_gc(),
                persist_moment,
            )
            .await;
        }
    }
}
