use my_sqlite::macros::*;

/// One persisted partition row in the SQLite backend. `content` is
/// `base64(zstd(JSON array of the partition's rows))` — base64 because
/// my-sqlite has no BLOB column type (a binary zstd blob is not valid UTF-8).
#[derive(TableSchema, InsertDbEntity, UpdateDbEntity, SelectDbEntity, Debug)]
pub struct MyNoSqlPartitionDto {
    #[primary_key(0)]
    pub table_name: String,
    #[primary_key(1)]
    pub partition_key: String,
    pub content: String,
}

#[derive(WhereDbModel, Debug)]
pub struct WherePartitionByTableName<'s> {
    pub table_name: &'s str,
}

#[derive(WhereDbModel, Debug)]
pub struct DeletePartitionWhereModel<'s> {
    pub table_name: &'s str,
    pub partition_key: &'s str,
}
