use std::collections::HashSet;

use base64::Engine;
use my_no_sql_sdk::core::db::DbTableAttributes;
use my_sqlite::sql_where::NoneWhereModel;
use my_sqlite::{SqlLiteConnection, SqlLiteConnectionBuilder};

use crate::operations::init::TableAttributeInitContract;
use crate::persist_repo::{LoadedPartition, LoadedTableAttrs};

use super::{
    DeletePartitionWhereModel, DeleteTableMetadataWhereModel, MyNoSqlPartitionDto,
    TableMetaDataDto, WherePartitionByTableName,
};

pub const PARTITIONS_TABLE: &str = "partitions";
pub const TABLES_METADATA_TABLE: &str = "tables_metadata";

pub struct SqlLiteRepo {
    sqlite: SqlLiteConnection,
}

impl SqlLiteRepo {
    pub async fn new(file_name: String) -> Self {
        println!("Connecting to SQLite: {}", file_name);
        Self {
            sqlite: SqlLiteConnectionBuilder::new(file_name)
                .create_table_if_no_exists::<MyNoSqlPartitionDto>(PARTITIONS_TABLE)
                .create_table_if_no_exists::<TableMetaDataDto>(TABLES_METADATA_TABLE)
                .build()
                .await
                .unwrap(),
        }
    }

    pub async fn save_partition(&self, table_name: &str, partition_key: &str, compressed: &[u8]) {
        let dto = MyNoSqlPartitionDto {
            table_name: table_name.to_string(),
            partition_key: partition_key.to_string(),
            content: base64::engine::general_purpose::STANDARD.encode(compressed),
        };
        self.sqlite
            .insert_or_update_db_entity(PARTITIONS_TABLE, &dto)
            .await
            .unwrap();
    }

    pub async fn delete_partition(&self, table_name: &str, partition_key: &str) {
        let where_model = DeletePartitionWhereModel {
            table_name,
            partition_key,
        };
        self.sqlite
            .delete_db_entity(PARTITIONS_TABLE, &where_model)
            .await
            .unwrap();
    }

    pub async fn clean_table_content(&self, table_name: &str) {
        let where_model = WherePartitionByTableName { table_name };
        self.sqlite
            .delete_db_entity(PARTITIONS_TABLE, &where_model)
            .await
            .unwrap();
    }

    /// Replaces the full persisted content of a table: writes every partition in
    /// `partitions` first (so the new data is durable), then deletes only the
    /// partitions that are no longer present. This avoids wiping a table's
    /// partitions before the replacements are written.
    pub async fn replace_table_partitions(
        &self,
        table_name: &str,
        partitions: Vec<(String, Vec<u8>)>,
    ) {
        let new_keys: HashSet<&str> = partitions.iter().map(|(pk, _)| pk.as_str()).collect();

        for (partition_key, compressed) in &partitions {
            self.save_partition(table_name, partition_key, compressed)
                .await;
        }

        let existing: Vec<MyNoSqlPartitionDto> = self
            .sqlite
            .query_rows(
                PARTITIONS_TABLE,
                Some(&WherePartitionByTableName { table_name }),
            )
            .await
            .unwrap();

        for dto in existing {
            if !new_keys.contains(dto.partition_key.as_str()) {
                self.delete_partition(table_name, &dto.partition_key).await;
            }
        }
    }

    pub async fn save_table_metadata(&self, table_name: &str, attr: &DbTableAttributes) {
        let dto = TableMetaDataDto::from_table_attr(table_name, attr);
        self.sqlite
            .insert_or_update_db_entity(TABLES_METADATA_TABLE, &dto)
            .await
            .unwrap();
    }

    pub async fn delete_table_metadata(&self, table_name: &str) {
        let dto = DeleteTableMetadataWhereModel { table_name };
        self.sqlite
            .delete_db_entity(TABLES_METADATA_TABLE, &dto)
            .await
            .unwrap();
    }

    pub async fn get_tables(&self) -> Vec<LoadedTableAttrs> {
        let dtos: Vec<TableMetaDataDto> = self
            .sqlite
            .query_rows(TABLES_METADATA_TABLE, NoneWhereModel::new())
            .await
            .unwrap();

        dtos.into_iter()
            .map(|dto| {
                let (table_name, attr) = TableAttributeInitContract::into(dto);
                LoadedTableAttrs { table_name, attr }
            })
            .collect()
    }

    /// Returns every persisted partition as `(table, partition, zstd bytes)`.
    /// `skip_errors` mirrors `SkipBrokenPartitions`: a row whose base64 content
    /// can not be decoded is skipped with a log when set, or is fatal otherwise.
    pub async fn load_all_partitions(&self, skip_errors: bool) -> Vec<LoadedPartition> {
        let partition_rows: Vec<MyNoSqlPartitionDto> = self
            .sqlite
            .query_rows(PARTITIONS_TABLE, NoneWhereModel::new())
            .await
            .unwrap();

        let mut result = Vec::with_capacity(partition_rows.len());

        for dto in partition_rows {
            match base64::engine::general_purpose::STANDARD.decode(dto.content.as_bytes()) {
                Ok(compressed) => {
                    result.push(LoadedPartition {
                        table_name: dto.table_name,
                        partition_key: dto.partition_key,
                        compressed,
                    });
                }
                Err(err) => {
                    let msg = format!(
                        "Can not base64-decode partition Table:{}. PartitionKey:{}. Err:{:?}",
                        dto.table_name, dto.partition_key, err
                    );
                    if skip_errors {
                        println!("{}", msg);
                    } else {
                        panic!("{}", msg);
                    }
                }
            }
        }

        result
    }

    /// Rebuilds the database file, reclaiming pages freed by deletes and
    /// defragmenting the storage. Takes an exclusive lock for the duration.
    pub async fn vacuum(&self) {
        self.sqlite
            .client
            .conn(|conn| conn.execute_batch("VACUUM"))
            .await
            .unwrap();
    }
}
