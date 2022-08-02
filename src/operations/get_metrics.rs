use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::db::DbTableWrapper;

pub struct DbTableMetrics {
    pub table_size: usize,
    pub partitions_amount: usize,
    pub expiration_index_records_amount: usize,
    pub records_amount: usize,
    pub last_update_time: DateTimeAsMicroseconds,
    pub last_persist_time: Option<DateTimeAsMicroseconds>,
    pub next_persist_time: Option<DateTimeAsMicroseconds>,
    pub persist_amount: usize,
    pub last_persist_duration: Vec<usize>,
}

pub async fn get_table_metrics(db_table_wrapper: &DbTableWrapper) -> DbTableMetrics {
    let table_read_access = db_table_wrapper.data.read().await;

    let persist_metrics = table_read_access.persist_markers.get_persist_metrics();

    DbTableMetrics {
        table_size: table_read_access.db_table.get_table_size(),
        partitions_amount: table_read_access.db_table.get_partitions_amount(),
        expiration_index_records_amount: table_read_access
            .db_table
            .get_expiration_index_rows_amount(),
        records_amount: table_read_access.db_table.get_rows_amount(),
        last_update_time: table_read_access.db_table.get_last_update_time(),
        last_persist_time: persist_metrics.last_persist_time,
        next_persist_time: persist_metrics.next_persist_time,
        persist_amount: persist_metrics.persist_amount,
        last_persist_duration: persist_metrics.last_persist_duration,
    }
}
