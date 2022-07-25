use std::sync::Arc;

use my_no_sql_core::db::DbRow;

pub enum WriteOperationResult {
    SingleRow(Arc<DbRow>),
    Empty,
}
