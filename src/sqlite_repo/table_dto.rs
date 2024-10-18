use my_sqlite::macros::*;

use crate::persist_operations::serializers::TableMetadataFileContract;

#[derive(TableSchema, InsertDbEntity, UpdateDbEntity, SelectDbEntity, Debug)]
pub struct TableAttributesDto {
    #[primary_key(0)]
    pub table_name: String,
    pub created_at: String,
    pub persist: bool,
    pub max_partitions_amount: Option<u64>,
    pub max_rows_per_partition_amount: Option<u64>,
}
#[derive(Debug, WhereDbModel)]
pub struct TableAttributesWhereModel<'s> {
    pub table_name: &'s str,
}

impl Into<TableMetadataFileContract> for TableAttributesDto {
    fn into(self) -> TableMetadataFileContract {
        TableMetadataFileContract {
            persist: self.persist,
            max_partitions_amount: self.max_partitions_amount.map(|x| x as usize),
            max_rows_per_partition_amount: self.max_rows_per_partition_amount.map(|x| x as usize),
            created: self.created_at.into(),
        }
    }
}
