use my_no_sql_sdk::core::db::DbRow;
use my_sqlite::macros::*;

#[derive(TableSchema, InsertDbEntity, UpdateDbEntity, SelectDbEntity, Debug)]
pub struct MyNoSqlEntityDto {
    #[primary_key(0)]
    pub table_name: String,
    #[primary_key(1)]
    pub partition_key: String,
    #[primary_key(2)]
    pub row_key: String,
    pub content: String,
}

impl MyNoSqlEntityDto {
    pub fn from_db_row(table_name: &str, db_row: &DbRow) -> Self {
        Self {
            table_name: table_name.to_string(),
            partition_key: db_row.get_partition_key().to_string(),
            row_key: db_row.get_row_key().to_string(),

            content: std::str::from_utf8(db_row.get_src_as_slice())
                .unwrap()
                .to_string(),
        }
    }
}

#[derive(WhereDbModel, Debug)]
pub struct WhereEntityByTableName<'s> {
    pub table_name: &'s str,
}

#[derive(WhereDbModel, Debug)]
pub struct WhereEntityByPartitionKey<'s> {
    pub table_name: &'s str,
    pub partition_key: &'s str,
}

#[derive(WhereDbModel, Debug)]
pub struct DeleteMyNoSqlEntityWhereModel<'s> {
    pub table_name: &'s str,
    pub partition_key: &'s str,
    pub row_key: &'s str,
}
