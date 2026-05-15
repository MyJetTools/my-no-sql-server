use my_sqlite::macros::*;

#[derive(TableSchema, InsertDbEntity, UpdateDbEntity, SelectDbEntity, Debug)]
pub struct MyNoSqlFileDto {
    #[primary_key(0)]
    #[generate_where_model(name = "WhereByTableModel", as_str)]
    #[generate_where_model(name = "WhereByFileName", as_str)]
    pub table_name: String,
    #[primary_key(1)]
    #[generate_where_model(name = "WhereByFileName", as_str)]
    pub file_name: String,
    pub content: String,
}
