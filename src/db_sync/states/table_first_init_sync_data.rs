use std::sync::Arc;

use crate::{data_readers::DataReader, db::DbTableWrapper};

pub struct TableFirstInitSyncData {
    pub db_table: Arc<DbTableWrapper>,
    pub data_reader: Arc<DataReader>,
}
