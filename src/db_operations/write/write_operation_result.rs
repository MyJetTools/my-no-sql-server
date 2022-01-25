use std::sync::Arc;

use crate::db::DbRow;

pub enum WriteOperationResult {
    SingleRow(Arc<DbRow>),
    Empty,
}
