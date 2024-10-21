use my_sqlite::sql_where::NoneWhereModel;
use my_sqlite::{SqlLiteConnection, SqlLiteConnectionBuilder};

use super::row_dto::*;

pub const FILES_TABLE: &str = "files";

pub struct SqlLiteRepo {
    sqlite: SqlLiteConnection,
}

impl SqlLiteRepo {
    pub async fn new(file_name: String) -> Self {
        Self {
            sqlite: SqlLiteConnectionBuilder::new(file_name)
                .create_table_if_no_exists::<MyNoSqlFileDto>(FILES_TABLE)
                .build()
                .await
                .unwrap(),
        }
    }

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
}
