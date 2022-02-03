use crate::{app::AppContext, db_operations::DbOperationError, json::JsonArrayBuilder};

use super::ReadOperationResult;

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
) -> Option<ReadOperationResult> {
    let result = app.multipart_list.get(multipart_id, amount).await?;

    let mut array_builder = JsonArrayBuilder::new();

    for db_row in result {
        array_builder.append_json_object(db_row.data.as_slice());
    }

    ReadOperationResult::RowsArray(array_builder.build()).into()
}
