use std::sync::Arc;

use crate::{app::AppContext, db::DbRow, db_operations::DbOperationError};

pub async fn start_read_all(app: &AppContext, table_name: &str) -> Result<i64, DbOperationError> {
    let db_table = super::table::get(app, table_name).await?;

    let entities = db_table.get_all_as_vec_dequeue().await;

    if entities.len() == 0 {
        return Ok(0);
    }

    let result = app.multipart_list.add(entities).await;

    Ok(result)
}

pub async fn get_next(
    app: &AppContext,
    multipart_id: i64,
    amount: usize,
) -> Option<Vec<Arc<DbRow>>> {
    return app.multipart_list.get(multipart_id, amount).await;
}