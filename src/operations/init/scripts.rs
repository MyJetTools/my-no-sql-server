use my_no_sql_sdk::core::db::{DbPartition, DbTable};
use my_no_sql_sdk::server::rust_extensions;

use crate::app::AppContext;

use super::{EntitiesInitReader, TableAttributeInitContract};

pub async fn init_tables(
    app: &AppContext,
    tables: Vec<impl TableAttributeInitContract>,
    mut entities_reader: impl EntitiesInitReader,
    save_to_db: bool,
) {
    for table_init_contract in tables {
        let (table_name, attr) = table_init_contract.into();
        let mut db_table = DbTable::new(table_name, attr);

        let db_rows = entities_reader.get_entities(db_table.name.as_str()).await;

        if let Some(db_rows) = db_rows {
            let by_partition =
                rust_extensions::grouped_data::group_to_btree_map(db_rows.into_iter(), |itm| {
                    itm.get_partition_key().to_string()
                });

            for (partition_key, entities) in by_partition {
                let mut db_partition = DbPartition::new(partition_key);
                for db_row in entities {
                    db_partition.insert_row(db_row);
                }

                db_table.restore_partition(db_partition);
            }
        }

        let db_table = crate::db_operations::write::table::init(app, db_table).await;

        if save_to_db {
            let table_snapshot = db_table.get_table_snapshot().await;

            println!("Migrating table: {}", db_table.name.as_str());
            app.repo
                .save_table_metadata(&db_table.name, &table_snapshot.attr)
                .await;

            crate::operations::persist::scripts::sync_table_snapshot(
                app,
                &db_table.name,
                table_snapshot,
            )
            .await;
        }
    }
}
