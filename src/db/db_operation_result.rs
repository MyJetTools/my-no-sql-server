use std::sync::Arc;

use super::DbRow;

pub enum DbOperationResult {
    Rows { rows: Option<Vec<Arc<DbRow>>> },
    Row { row: Arc<DbRow> },
}
