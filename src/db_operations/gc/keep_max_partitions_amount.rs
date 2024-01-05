use std::sync::Arc;

use my_no_sql_sdk::core::db::PartitionKey;
use my_no_sql_server_core::DbTableWrapper;
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{app::AppContext, db_operations::DbOperationError, db_sync::EventSource};

pub async fn keep_max_partitions_amount(
    app: &AppContext,
    db_table: &Arc<DbTableWrapper>,
    max_partitions_amount: usize,
    event_src: EventSource,
    persist_moment: DateTimeAsMicroseconds,
) -> Result<(), DbOperationError> {
    super::super::check_app_states(app)?;

    let partitions_to_gc: Option<Vec<PartitionKey>> = {
        let read_access = db_table.data.read().await;

        let result = read_access
            .partitions
            .get_partitions_to_gc_by_max_amount(max_partitions_amount);

        if let Some(result) = result {
            Some(result.into_iter().map(|p| p.clone()).collect())
        } else {
            None
        }
    };

    if let Some(partitions_to_gc) = partitions_to_gc {
        super::super::write::delete_partitions(
            app,
            db_table,
            partitions_to_gc.iter().map(|itm| itm.as_str()),
            event_src,
            persist_moment,
            DateTimeAsMicroseconds::now(),
        )
        .await?;
    }

    Ok(())
}
