use my_no_sql_sdk::core::db::{DbTableAttributes, DbTableName, PartitionKey};
use my_sqlite::sql_where::NoneWhereModel;
use my_sqlite::{SqlLiteConnection, SqlLiteConnectionBuilder};

use super::{
    DeleteMyNoSqlEntityWhereModel, DeleteTableMetadataWhereModel, MyNoSqlEntityDto,
    TableMetaDataDto, WhereEntityByPartitionKey, WhereEntityByTableName,
};

//pub const FILES_TABLE: &str = "files";
pub const ENTITIES_TABLE: &str = "entities";
pub const TABLES_METADATA_TABLE: &str = "tables_metadata";

pub struct SqlLiteRepo {
    sqlite: SqlLiteConnection,
}

impl SqlLiteRepo {
    pub async fn new(file_name: String) -> Self {
        println!("Connecting to SQLite: {}", file_name);
        Self {
            sqlite: SqlLiteConnectionBuilder::new(file_name)
                // .create_table_if_no_exists::<MyNoSqlFileDto>(FILES_TABLE)
                .create_table_if_no_exists::<MyNoSqlEntityDto>(ENTITIES_TABLE)
                .create_table_if_no_exists::<TableMetaDataDto>(TABLES_METADATA_TABLE)
                .build()
                .await
                .unwrap(),
        }
    }

    pub async fn save_entities(&self, entities: &[MyNoSqlEntityDto]) {
        self.sqlite
            .bulk_insert_or_update(entities, ENTITIES_TABLE)
            .await
            .unwrap();
    }

    pub async fn get_all_entities(&self) -> Vec<MyNoSqlEntityDto> {
        self.sqlite
            .query_rows(ENTITIES_TABLE, NoneWhereModel::new())
            .await
            .unwrap()
    }

    pub async fn delete_entity(
        &self,
        table_name: &DbTableName,
        partition_key: &str,
        row_key: &str,
    ) {
        let where_model = DeleteMyNoSqlEntityWhereModel {
            table_name: table_name.as_str(),
            partition_key: partition_key,
            row_key,
        };
        self.sqlite
            .delete_db_entity(ENTITIES_TABLE, &where_model)
            .await
            .unwrap();
    }

    pub async fn clean_table_content(&self, table_name: &DbTableName) {
        let where_model = WhereEntityByTableName {
            table_name: table_name.as_str(),
        };
        self.sqlite
            .delete_db_entity(ENTITIES_TABLE, &where_model)
            .await
            .unwrap();
    }

    pub async fn clean_partition_content(
        &self,
        table_name: &DbTableName,
        partition_key: &PartitionKey,
    ) {
        let where_model = WhereEntityByPartitionKey {
            table_name: table_name.as_str(),
            partition_key: partition_key.as_str(),
        };
        self.sqlite
            .delete_db_entity(ENTITIES_TABLE, &where_model)
            .await
            .unwrap();
    }

    pub async fn save_table_metadata(&self, table_name: &DbTableName, attr: &DbTableAttributes) {
        let dto = TableMetaDataDto::from_table_attr(table_name.as_str(), &attr);

        self.sqlite
            .insert_or_update_db_entity(TABLES_METADATA_TABLE, &dto)
            .await
            .unwrap();
    }

    pub async fn delete_table_metadata(&self, table_name: &DbTableName) {
        let dto = DeleteTableMetadataWhereModel {
            table_name: table_name.as_str(),
        };

        self.sqlite
            .delete_db_entity(TABLES_METADATA_TABLE, &dto)
            .await
            .unwrap();
    }

    pub async fn get_tables(&self) -> Vec<TableMetaDataDto> {
        self.sqlite
            .query_rows(TABLES_METADATA_TABLE, NoneWhereModel::new())
            .await
            .unwrap()
    }

    /*
    pub async fn save_file(&self, table_name: &str, file_name: &str, content: String) {
        let table = MyNoSqlFileDto {
            table_name: table_name.to_string(),
            file_name: file_name.to_string(),
            content,
        };

        self.sqlite
            .insert_or_update_db_entity(FILES_TABLE, &table)
            .await
            .unwrap();
    }

    pub async fn save_files(&self, dto: &[MyNoSqlFileDto]) {
        self.sqlite
            .bulk_insert_or_update(dto, FILES_TABLE)
            .await
            .unwrap();
    }

    pub async fn get_files(&self) -> Vec<MyNoSqlFileDto> {
        self.sqlite
            .query_rows(FILES_TABLE, NoneWhereModel::new())
            .await
            .unwrap()
    }

    pub async fn delete_file(&self, table_name: &str, file_name: &str) {
        let where_model = WhereByFileName {
            table_name,
            file_name,
        };
        self.sqlite
            .delete_db_entity(FILES_TABLE, &where_model)
            .await
            .unwrap();
    }
     */
}
