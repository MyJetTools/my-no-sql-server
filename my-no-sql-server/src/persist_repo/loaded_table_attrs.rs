use my_no_sql_sdk::core::db::{DbTableAttributes, DbTableName};

use crate::operations::init::TableAttributeInitContract;

/// Backend-neutral table descriptor returned by `PersistRepo::get_tables`.
/// Both the SQLite and the Files backend produce this, so the init path
/// (`init_tables`) is backend-agnostic.
pub struct LoadedTableAttrs {
    pub table_name: DbTableName,
    pub attr: DbTableAttributes,
}

impl TableAttributeInitContract for LoadedTableAttrs {
    fn into(self) -> (DbTableName, DbTableAttributes) {
        (self.table_name, self.attr)
    }
}
