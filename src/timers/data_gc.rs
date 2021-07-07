use std::{sync::Arc, time::Duration};

use crate::{
    app::{logs::SystemProcess, AppServices},
    db_transactions::TransactionAttributes,
};

pub async fn start(app: Arc<AppServices>) {
    let delay = Duration::from_secs(10);
    app.logs
        .add_info(
            None,
            SystemProcess::System,
            "Timer dead data readers gc".to_string(),
            "Started".to_string(),
        )
        .await;

    let transaction_attr = TransactionAttributes {
        headers: None,
        event_source: crate::db_transactions::EventSource::GarbageCollector,
        locations: vec![app.settings.location.to_string()],
        sync_period: crate::db_transactions::DataSynchronizationPeriod::Sec1,
    };

    loop {
        tokio::time::sleep(delay).await;

        let tables = app.db.get_tables().await;

        for db_table in tables {
            let attr = db_table.get_attributes().await;

            if attr.max_partitions_amount.is_none() {
                continue;
            }

            let max_partitions_amount = attr.max_partitions_amount.unwrap();

            crate::db_operations::gc::keep_max_partitions_amount(
                app.as_ref(),
                db_table,
                max_partitions_amount,
                Some(transaction_attr.clone()),
            )
            .await;
        }
    }
}
