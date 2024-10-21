use my_no_sql_server_core::db_snapshots::DbTableSnapshot;

use crate::{
    persist_io::TableFile,
    persist_operations::serializers::TableMetadataFileContract,
    sqlite_repo::{MyNoSqlFileDto, SqlLiteRepo},
};

pub async fn init_new_instance_table(
    repo: &SqlLiteRepo,
    table_name: &str,
    db_table_snapshot: DbTableSnapshot,
) {
    let mut items_to_insert = Vec::new();
    let table_attributes: TableMetadataFileContract = (&db_table_snapshot.attr).into();

    items_to_insert.push(MyNoSqlFileDto {
        table_name: table_name.to_string(),
        file_name: TableFile::TableAttributes
            .get_file_name()
            .as_str()
            .to_string(),
        content: String::from_utf8(table_attributes.to_vec()).unwrap(),
    });

    for partition in db_table_snapshot.by_partition {
        let content = partition.db_rows_snapshot.as_json_array();
        let dto = MyNoSqlFileDto {
            table_name: table_name.to_string(),
            file_name: TableFile::DbPartition(partition.partition_key)
                .get_file_name()
                .as_str()
                .to_string(),
            content: String::from_utf8(content.build()).unwrap(),
        };

        items_to_insert.push(dto);
    }

    for chunk in items_to_insert.chunks(1000) {
        repo.save_files(chunk).await;
    }
}
