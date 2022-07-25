use std::sync::Arc;

use my_no_sql_core::db::DbTable;

use crate::data_readers::DataReader;

pub struct TableFirstInitSyncData {
    pub db_table: Arc<DbTable>,
    pub data_reader: Arc<DataReader>,
}
