use std::collections::HashMap;

use crate::{
    app::AppServices,
    db::DbTable,
    db_transactions::{TransactionAttributes, TransactionEvent},
};

pub async fn clean_and_keep_max_partitions_amount(
    app: &AppServices,
    db_table: &DbTable,
    max_partitions_amount: usize,
    attr: Option<TransactionAttributes>,
) {
    let partitions_amount = db_table.get_partitions_amount().await;

    if partitions_amount <= max_partitions_amount {
        return;
    }

    let mut write_access = db_table.data.write().await;

    let gced_partitions_result = write_access.gc_partitions(max_partitions_amount).await;

    if let Some(attr) = attr {
        app.dispatch_event(TransactionEvent::DeletePartitions {
            table_name: db_table.name.to_string(),
            attr,
            partitions: gced_partitions_result,
        })
        .await;
    }
}

pub async fn clean_and_keep_max_records(
    app: &AppServices,
    db_table: &DbTable,
    partition_key: &str,
    max_rows_amount: usize,
    attr: Option<TransactionAttributes>,
) {
    let rows_amount = db_table.get_partition_rows_amount(partition_key).await;

    if rows_amount.is_none() {
        return;
    }

    if rows_amount.unwrap() <= max_rows_amount {
        return;
    }

    let mut write_access = db_table.data.write().await;

    let partition = write_access.get_partition_mut(partition_key);

    if partition.is_none() {
        return;
    }

    let partition = partition.unwrap();

    let gced_rows = partition.gc_rows(max_rows_amount);

    let mut sync = HashMap::new();
    sync.insert(partition_key.to_string(), gced_rows);

    if let Some(attr) = attr {
        app.dispatch_event(TransactionEvent::DeleteRows {
            table_name: db_table.name.to_string(),
            attr,
            rows: sync,
        })
        .await;
    }
}
