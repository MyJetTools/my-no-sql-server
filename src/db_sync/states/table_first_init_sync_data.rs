use std::sync::Arc;

use crate::{data_readers::DataReader, db::DbTable};

pub struct TableFirstInitSyncData {
    pub db_table: Arc<DbTable>,
    pub data_reader: Arc<DataReader>,
}
