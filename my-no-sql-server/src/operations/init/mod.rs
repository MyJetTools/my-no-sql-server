mod load_tables;
use std::sync::Arc;

pub use load_tables::*;
mod entities_from_sqlite_reader;
mod from_other_instance;
mod scripts;
use my_no_sql_sdk::core::db::{DbRow, DbTableAttributes, DbTableName};

mod init_state;
pub use init_state::*;

pub trait TableAttributeInitContract {
    fn into(self) -> (DbTableName, DbTableAttributes);
}

#[async_trait::async_trait]
pub trait EntitiesInitReader {
    async fn get_entities(&mut self, table_name: &str) -> Option<Vec<Arc<DbRow>>>;
}
