use my_no_sql_sdk::core::db::{DbTableName, PartitionKey};

use crate::app::AppContext;

pub async fn save_partition(
    app: &AppContext,
    db_table_name: &DbTableName,
    partition_key: PartitionKey,
) {
    let db_table = match app.db.get_table(db_table_name.as_str()).await {
        Some(db_table) => db_table,
        None => {
            super::scripts::delete_table(app, db_table_name).await;
            return;
        }
    };

    match db_table
        .get_partition_snapshot(partition_key.as_str())
        .await
    {
        Some(snapshot) => {
            super::scripts::sync_partition_snapshot(app, db_table_name, &partition_key, snapshot)
                .await;
        }
        None => {
            super::scripts::delete_partition(app, db_table_name, &partition_key).await;
            return;
        }
    };
}
