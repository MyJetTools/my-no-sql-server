use std::sync::Arc;

use my_no_sql_server_core::DbTableWrapper;
use rust_extensions::date_time::DateTimeAsMicroseconds;

pub fn update_partition_expiration_time(
    db_table: &Arc<DbTableWrapper>,
    partition_key: &String,
    set_expiration_time: Option<DateTimeAsMicroseconds>,
) {
    let partition_key = partition_key.to_string();

    let db_table = db_table.clone();

    tokio::spawn(async move {
        let mut table_data = db_table.data.write().await;

        if let Some(db_partition) = table_data.get_partition_mut(&partition_key) {
            db_partition.expires = set_expiration_time;
        }
    });
}
