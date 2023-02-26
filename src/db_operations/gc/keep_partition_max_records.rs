use std::collections::HashMap;

use my_no_sql_server_core::DbTableWrapper;
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{app::AppContext, db_operations::DbOperationError, db_sync::EventSource};

pub async fn execute(
    app: &AppContext,
    db_table: &DbTableWrapper,
    partition_key: &String,
    max_rows_amount: usize,
    event_source: EventSource,
    persist_moment: DateTimeAsMicroseconds,
) -> Result<(), DbOperationError> {
    super::super::check_app_states(app)?;

    let rows_to_gc: Vec<String> = {
        let table_data = db_table.data.read().await;
        let partition = table_data.get_partition(partition_key);

        if partition.is_none() {
            return Ok(());
        }

        let db_rows = partition
            .unwrap()
            .rows
            .get_rows_to_gc_by_max_amount(max_rows_amount);

        if db_rows.is_none() {
            return Ok(());
        }

        let db_rows = db_rows.unwrap();

        db_rows.into_iter().map(|r| r.row_key.to_string()).collect()
    };

    let mut row_to_delete = HashMap::new();
    row_to_delete.insert(partition_key.to_string(), rows_to_gc);

    super::super::write::bulk_delete(
        app,
        db_table,
        row_to_delete,
        event_source,
        persist_moment,
        DateTimeAsMicroseconds::now(),
    )
    .await?;

    Ok(())
}
