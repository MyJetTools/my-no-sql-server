use std::sync::Arc;

use my_no_sql_server_core::DbTableWrapper;
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::app::AppContext;

pub fn update_partition_expiration_time(
    app: &Arc<AppContext>,
    db_table: &Arc<DbTableWrapper>,
    partition_key: &str,
    set_expiration_time: Option<DateTimeAsMicroseconds>,
) {
    let partition_key = partition_key.to_string();

    let db_table = db_table.clone();

    let app = app.clone();

    tokio::spawn(async move {
        let mut table_data = db_table.data.write().await;

        if let Some(db_partition) = table_data.get_partition_mut(&partition_key) {
            db_partition.expires = set_expiration_time;
        }

        let mut sync_moment = DateTimeAsMicroseconds::now();
        sync_moment.add_minutes(5);

        app.persist_markers
            .persist_partition(&table_data, &partition_key, sync_moment)
            .await;
    });
}
