use std::sync::Arc;

use my_no_sql_sdk::core::rust_extensions::date_time::DateTimeAsMicroseconds;
use my_no_sql_sdk::server::DbTableWrapper;

use crate::app::AppContext;

pub fn update_rows_expiration_time<'s, TRowKeys: Iterator<Item = &'s str>>(
    app: &Arc<AppContext>,
    db_table: &Arc<DbTableWrapper>,
    partition_key: &str,
    row_keys: TRowKeys,
    set_expiration_time: Option<DateTimeAsMicroseconds>,
) {
    let partition_key = partition_key.to_string();

    let row_keys = row_keys.map(|itm| itm.to_owned()).collect::<Vec<_>>();

    let db_table = db_table.clone();

    let app = app.clone();

    tokio::spawn(async move {
        let mut table_data = db_table.data.write().await;

        let mut updated_db_rows = Vec::new();

        let mut db_partition_key = None;

        if let Some(db_partition) = table_data.get_partition_mut(&partition_key) {
            for row_key in row_keys {
                let db_row = db_partition
                    .rows
                    .update_expiration_time(&row_key, set_expiration_time);

                if let Some(db_row) = db_row {
                    updated_db_rows.push(db_row);

                    if db_partition_key.is_none() {
                        db_partition_key = Some(db_partition.partition_key.clone());
                    }
                }
            }
        }

        if let Some(db_partition_key) = db_partition_key {
            let mut sync_moment = DateTimeAsMicroseconds::now();
            sync_moment.add_seconds(10);

            app.persist_markers
                .persist_rows(
                    &table_data.name,
                    &db_partition_key,
                    sync_moment,
                    updated_db_rows.iter(),
                )
                .await;
        }
    });
}
