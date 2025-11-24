use std::{collections::HashMap, sync::Arc};

use my_no_sql_sdk::core::{db::DbRow, db_json_entity::DbJsonEntity};
use my_no_sql_sdk::server::rust_extensions;

use crate::sqlite_repo::MyNoSqlEntityDto;

use super::EntitiesInitReader;

pub struct EntitiesFromSqliteReader {
    by_table: HashMap<String, Vec<MyNoSqlEntityDto>>,
    skip_errors: bool,
}

impl EntitiesFromSqliteReader {
    pub fn new(entities: Vec<MyNoSqlEntityDto>, skip_errors: bool) -> Self {
        Self {
            skip_errors,
            by_table: rust_extensions::grouped_data::group_to_hash_map(
                entities.into_iter(),
                |itm| &itm.table_name,
            ),
        }
    }
}

#[async_trait::async_trait]
impl EntitiesInitReader for EntitiesFromSqliteReader {
    async fn get_entities(&mut self, table_name: &str) -> Option<Vec<Arc<DbRow>>> {
        let dtos = self.by_table.remove(table_name)?;

        let mut result = Vec::with_capacity(dtos.len());

        for dto in dtos {
            match DbJsonEntity::restore_into_db_row(dto.content.into_bytes()) {
                Ok(db_row) => result.push(Arc::new(db_row)),
                Err(err) => {
                    if self.skip_errors {
                        println!(
                            "Can not restore row Table:{}. PartitionKey: {} RowKey: {}.  Err: {:?}",
                            table_name, dto.partition_key, dto.row_key, err
                        );
                    } else {
                        panic!(
                            "Can not restore row Table:{}. PartitionKey: {} RowKey: {}.  Err: {:?}",
                            table_name, dto.partition_key, dto.row_key, err
                        );
                    }
                }
            }
        }

        Some(result)
    }
}
