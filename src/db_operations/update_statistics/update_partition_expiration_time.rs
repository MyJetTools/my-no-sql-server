use std::sync::Arc;

use my_no_sql_sdk::core::rust_extensions::date_time::DateTimeAsMicroseconds;
use my_no_sql_server_core::DbTableWrapper;

pub fn update_partition_expiration_time(
    db_table: &Arc<DbTableWrapper>,
    partition_key: String,
    set_expiration_time: Option<DateTimeAsMicroseconds>,
) {
    let db_table = db_table.clone();

    tokio::spawn(async move {
        let mut table_data = db_table.data.write().await;

        if let Some(db_partition) = table_data.get_partition_mut(partition_key.as_str()) {
            db_partition.expires = set_expiration_time;
        }
    });
}
