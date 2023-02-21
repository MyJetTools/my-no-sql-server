use std::sync::Arc;

use my_no_sql_server_core::DbTableWrapper;
use rust_extensions::date_time::DateTimeAsMicroseconds;

pub fn update_rows_expiration_time<'s, TRowKeys: Iterator<Item = &'s String>>(
    db_table: &Arc<DbTableWrapper>,
    partition_key: &str,
    row_keys: TRowKeys,
    set_expiration_time: Option<DateTimeAsMicroseconds>,
) {
    let partition_key = partition_key.to_string();

    let row_keys = row_keys.map(|itm| itm.to_owned()).collect::<Vec<_>>();

    let db_table = db_table.clone();

    tokio::spawn(async move {
        let mut table_data = db_table.data.write().await;

        if let Some(db_partition) = table_data.get_partition_mut(&partition_key) {
            for row_key in row_keys {
                if let Some(db_row) = db_partition.get_row_and_clone(&row_key) {
                    db_partition
                        .rows
                        .update_expiration_time(db_row, set_expiration_time);
                }
            }
        }
    });
}
