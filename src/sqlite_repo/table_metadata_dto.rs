use my_no_sql_sdk::core::db::{DbTableAttributes, DbTableName};
use my_no_sql_server_core::rust_extensions::date_time::DateTimeAsMicroseconds;
use my_sqlite::macros::*;

use crate::operations::init::TableAttributeInitContract;
#[derive(TableSchema, InsertDbEntity, UpdateDbEntity, SelectDbEntity, Debug)]
pub struct TableMetaDataDto {
    #[primary_key(0)]
    pub table_name: String,
    pub max_partitions_amount: i64,
    pub max_rows_per_partition_amount: i64,
    pub persist: bool,
    pub created: i64,
}

impl TableMetaDataDto {
    pub fn from_table_attr(table_name: &str, attr: &DbTableAttributes) -> Self {
        Self {
            table_name: table_name.to_string(),
            max_partitions_amount: attr.max_partitions_amount.unwrap_or(0) as i64,
            max_rows_per_partition_amount: attr.max_rows_per_partition_amount.unwrap_or(0) as i64,
            persist: attr.persist,
            created: attr.created.unix_microseconds,
        }
    }
}

impl TableAttributeInitContract for TableMetaDataDto {
    fn into(self) -> (DbTableName, DbTableAttributes) {
        let attr = DbTableAttributes {
            max_partitions_amount: if self.max_partitions_amount <= 0 {
                None
            } else {
                Some(self.max_partitions_amount as usize)
            },
            max_rows_per_partition_amount: if self.max_rows_per_partition_amount <= 0 {
                None
            } else {
                Some(self.max_rows_per_partition_amount as usize)
            },
            persist: self.persist,
            created: DateTimeAsMicroseconds::new(self.created),
        };

        (self.table_name.into(), attr)
    }
}

#[derive(Debug, WhereDbModel)]
pub struct DeleteTableMetadataWhereModel<'s> {
    pub table_name: &'s str,
}
