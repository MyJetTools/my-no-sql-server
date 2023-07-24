use std::sync::Arc;

use my_no_sql_server_core::DbTableWrapper;

use crate::{app::AppContext, db_operations::DbOperationError};

pub async fn get_partitions(
    app: &AppContext,
    db_table: &Arc<DbTableWrapper>,
    limit: Option<usize>,
    skip: Option<usize>,
) -> Result<(usize, Vec<String>), DbOperationError> {
    super::super::super::check_app_states(app)?;

    let table_data = db_table.data.read().await;

    let count = table_data.partitions.len();

    let items = table_data.partitions.get_all().iter().map(|itm| itm.0);

    let result = crate::db_operations::read::read_filter::filter_it(items, limit, skip);

    match result {
        Some(items) => Ok((count, items.iter().map(|itm| itm.to_string()).collect())),
        None => Ok((count, vec![])),
    }
}
