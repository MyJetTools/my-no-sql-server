use my_no_sql_sdk::core::db::DbTable;

pub struct SyncTableData {
    pub table_name: String,
    pub persist: bool,
}

impl SyncTableData {
    pub fn new(table_data: &DbTable) -> Self {
        Self {
            table_name: table_data.name.to_string(),
            persist: table_data.attributes.persist,
        }
    }
}
