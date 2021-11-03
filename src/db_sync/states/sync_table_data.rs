use crate::db::DbTable;

pub struct SyncTableData {
    pub table_name: String,
    pub persist: bool,
}

impl SyncTableData {
    pub fn new(db_table: &DbTable) -> Self {
        Self {
            table_name: db_table.name.to_string(),
            persist: db_table.get_persist(),
        }
    }
}
