use std::{collections::BTreeMap, sync::Arc};

use my_no_sql_sdk::core::db::{DbPartition, DbTable};
use my_no_sql_server_core::{rust_extensions::date_time::DateTimeAsMicroseconds, DbTableWrapper};

use crate::{
    app::AppContext, persist_io::TableFile,
    persist_operations::serializers::TableMetadataFileContract,
};

pub async fn init_from_another_instance(app: &Arc<AppContext>, url: &str) {
    let tables = super::load_tables(url).await;

    println!("Loaded {} tables from instance: {}", tables.len(), url);

    for table in tables {
        println!("Initializing table {}", table.name);
        app.persist_io.create_table_folder(&table.name).await;

        let table_file = TableFile::TableAttributes;

        let table_metadata_contract = TableMetadataFileContract {
            persist: table.persist.unwrap_or(true),
            max_partitions_amount: table.max_partitions_amount.map(|itm| itm as usize),
            max_rows_per_partition_amount: table
                .max_rows_per_partition_amount
                .map(|itm| itm as usize),
            created: Some(DateTimeAsMicroseconds::now().to_rfc3339()),
        };

        app.persist_io
            .save_table_file(&table.name, &table_file, table_metadata_contract.to_vec())
            .await;

        let table_content = super::load_rows(url, &table.name).await;

        let mut by_partition: BTreeMap<String, Vec<Arc<my_no_sql_sdk::core::db::DbRow>>> =
            BTreeMap::new();

        for row in table_content {
            if let Some(db_partition) = by_partition.get_mut(row.get_partition_key()) {
                db_partition.push(row);
            } else {
                by_partition.insert(row.get_partition_key().to_string(), vec![row]);
            }
        }

        let mut db_table = DbTable::new(table.name.to_string(), table_metadata_contract.into());

        for (partition_key, rows) in by_partition {
            let mut db_partition = DbPartition::new(partition_key);
            db_partition.insert_or_replace_rows_bulk(rows.as_slice());
            db_table.init_partition(db_partition);
        }

        let db_table = DbTableWrapper::new(db_table);

        let table_snapshot = db_table.get_table_snapshot().await;

        for partition_snapshot in table_snapshot.by_partition {
            let content = partition_snapshot.db_rows_snapshot.as_json_array();

            let table_file = TableFile::DbPartition(partition_snapshot.partition_key.clone());
            println!(
                "Persisting table {} file {}",
                table.name,
                table_file.get_file_name().as_str()
            );
            app.persist_io
                .save_table_file(&table.name, &table_file, content.build())
                .await;
        }

        let mut table_access = app.db.tables.write().await;
        table_access.insert(db_table.name.to_string(), db_table);
    }
}
