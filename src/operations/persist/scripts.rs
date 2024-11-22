use std::sync::Arc;

use my_no_sql_sdk::core::db::{DbRow, DbTableName, PartitionKey};
use my_no_sql_server_core::db_snapshots::{DbPartitionSnapshot, DbTableSnapshot};

use crate::{app::AppContext, sqlite_repo::MyNoSqlEntityDto};

pub async fn delete_partition(
    app: &AppContext,
    table_name: &DbTableName,
    partition_key: &PartitionKey,
) {
    app.repo
        .clean_partition_content(table_name, partition_key)
        .await;
}

pub async fn delete_table(app: &AppContext, table_name: &DbTableName) {
    app.repo.clean_table_content(table_name).await;
    app.repo.delete_table_metadata(table_name).await;
}

pub async fn sync_table_snapshot(
    app: &AppContext,
    table_name: &DbTableName,
    table_snapshot: DbTableSnapshot,
) {
    app.repo.clean_table_content(table_name).await;

    let mut rows_to_save = Vec::new();
    for partition_snapshot in table_snapshot.by_partition {
        for db_row in partition_snapshot.db_rows_snapshot.db_rows.iter() {
            rows_to_save.push(MyNoSqlEntityDto::from_db_row(table_name.as_str(), db_row));

            if rows_to_save.len() >= super::SAVE_ENTITIES_BATCH_SIZE {
                app.repo.save_entities(&rows_to_save).await;
                rows_to_save.clear();
            }
        }
    }

    if rows_to_save.len() > 0 {
        app.repo.save_entities(&rows_to_save).await;
    }
}

pub async fn sync_partition_snapshot(
    app: &AppContext,
    table_name: &DbTableName,
    partition_key: &PartitionKey,
    partition_snapshot: DbPartitionSnapshot,
) {
    app.repo
        .clean_partition_content(table_name, partition_key)
        .await;

    save_rows(
        app,
        table_name,
        &partition_snapshot.db_rows_snapshot.db_rows,
    )
    .await;
}

pub async fn delete_rows(app: &AppContext, table_name: &DbTableName, db_rows: &[Arc<DbRow>]) {
    for db_row in db_rows {
        app.repo
            .delete_entity(table_name, db_row.get_partition_key(), db_row.get_row_key())
            .await;
    }
}

pub async fn save_rows(app: &AppContext, table_name: &DbTableName, db_rows: &[Arc<DbRow>]) {
    let mut rows_to_save = Vec::new();

    for db_row in db_rows.iter() {
        rows_to_save.push(MyNoSqlEntityDto::from_db_row(table_name.as_str(), db_row));

        if rows_to_save.len() >= super::SAVE_ENTITIES_BATCH_SIZE {
            app.repo.save_entities(&rows_to_save).await;
            rows_to_save.clear();
        }
    }

    if rows_to_save.len() > 0 {
        app.repo.save_entities(&rows_to_save).await;
    }
}
