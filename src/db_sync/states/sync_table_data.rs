use my_no_sql_sdk::core::db::{DbTable, DbTableName};

pub struct SyncTableData {
    pub table_name: DbTableName,
}

impl SyncTableData {
    pub fn new(table_data: &DbTable) -> Self {
        Self {
            table_name: table_data.name.clone(),
        }
    }
}
