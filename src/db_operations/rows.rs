use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

use crate::{
    app::AppServices,
    date_time::MyDateTime,
    db::{DbOperationFail, DbRow, DbTable},
    db_entity::{DbEntity, DbEntityParseFail},
    db_transactions::{TransactionAttributes, TransactionEvent},
};

pub async fn insert(
    app: &AppServices,
    db_table: &DbTable,
    payload: &[u8],
    attr: Option<TransactionAttributes>,
) -> Result<(), DbOperationFail> {
    let db_entity = DbEntity::parse(payload)?;
    let now = MyDateTime::utc_now();
    let mut table_write_access = db_table.data.write().await;

    let db_partition = table_write_access
        .get_or_create_partition_and_update_last_access(db_entity.partition_key.as_str(), now);

    let db_row = Arc::new(DbRow::form_db_entity(&db_entity));

    let inserted = db_partition.insert(db_row.clone());

    if inserted {
        if let Some(attr) = attr {
            app.dispatch_event(TransactionEvent::update_row(
                db_table,
                attr,
                db_entity.partition_key.as_str(),
                db_row,
            ))
            .await;
        }
    }

    return Ok(());
}

pub async fn insert_or_replace(
    app: &AppServices,
    db_table: &DbTable,
    payload: &[u8],
    attr: Option<TransactionAttributes>,
) -> Result<(), DbOperationFail> {
    let db_entity = DbEntity::parse(payload)?;
    let now = MyDateTime::utc_now();

    let mut table_write_access = db_table.data.write().await;

    let db_partition = table_write_access
        .get_or_create_partition_and_update_last_access(db_entity.partition_key.as_str(), now);

    let db_row = Arc::new(DbRow::form_db_entity(&db_entity));

    db_partition.insert_or_replace(db_row.clone());

    if let Some(attr) = attr {
        app.dispatch_event(TransactionEvent::update_row(
            db_table,
            attr,
            db_entity.partition_key.as_str(),
            db_row.clone(),
        ))
        .await;
    }

    return Ok(());
}

pub async fn replace(
    app: &AppServices,
    db_table: &DbTable,
    payload: &[u8],
    attr: Option<TransactionAttributes>,
) -> Result<(), DbOperationFail> {
    let entity = DbEntity::parse(payload)?;

    if entity.time_stamp.is_none() {
        return Err(DbOperationFail::DbEntityParseFail(
            DbEntityParseFail::TimeStampFieldRequires,
        ));
    }

    let entity_time_stamp = entity.time_stamp.unwrap();

    let mut write_access = db_table.data.write().await;

    let now = MyDateTime::utc_now();

    let db_partition =
        write_access.get_partition_and_update_last_access_mut(entity.partition_key.as_str(), now);

    if db_partition.is_none() {
        return Err(DbOperationFail::RecordNotFound);
    }

    let db_partition = db_partition.unwrap();

    let db_row = db_partition.get_row_and_update_last_time(entity.row_key.as_str(), now);

    if db_row.is_none() {
        return Err(DbOperationFail::RecordNotFound);
    }

    let db_row = db_row.unwrap();

    if !db_row.time_stamp.equals_to(entity_time_stamp) {
        return Err(DbOperationFail::OptimisticConcurencyUpdateFails);
    }

    let db_row = Arc::new(DbRow::form_db_entity(&entity));

    db_partition.insert_or_replace(db_row.clone());

    if let Some(attr) = attr {
        app.dispatch_event(TransactionEvent::UpdateRow {
            table: db_table.into(),
            attr,
            partition_key: entity.partition_key.to_string(),
            row: db_row,
        })
        .await
    }

    Ok(())
}

pub async fn clean_table(
    app: &AppServices,
    db_table: &DbTable,
    attr: Option<TransactionAttributes>,
) {
    let mut table_write_access = db_table.data.write().await;

    let cleaned = table_write_access.clear();

    if cleaned {
        if let Some(attr) = attr {
            app.dispatch_event(TransactionEvent::CleanTable {
                table: db_table.into(),
                attr,
            })
            .await
        }
    }
}

pub async fn delete_rows(
    app: &AppServices,
    db_table: &DbTable,
    partition_key: String,
    row_keys: Vec<String>,
    attr: Option<TransactionAttributes>,
) {
    let mut table_write_access = db_table.data.write().await;

    let mut removed_rows = Vec::new();

    let db_partition = table_write_access.get_partition_mut(partition_key.as_str());

    if db_partition.is_none() {
        return;
    }

    for row_key in &row_keys {
        let delete_row_result = table_write_access.delete_row(partition_key.as_str(), row_key);

        if let Some(deleted_row) = delete_row_result {
            removed_rows.push(deleted_row.row_key.to_string());
        }
    }

    if removed_rows.len() > 0 {
        if let Some(attr) = attr {
            let mut rows = HashMap::new();
            rows.insert(partition_key, removed_rows);

            app.dispatch_event(TransactionEvent::DeleteRows {
                table: db_table.into(),
                rows,
                attr,
            })
            .await
        }
    }
}

pub async fn delete_partitions(
    app: &AppServices,
    db_table: &DbTable,
    partition_keys: Vec<String>,
    attr: Option<TransactionAttributes>,
) {
    let mut table_write_access = db_table.data.write().await;

    let mut removed_partitions = Vec::new();

    for partition_key in &partition_keys {
        let remove_partition_result = table_write_access.remove_partition(partition_key);

        if remove_partition_result.is_some() {
            removed_partitions.push(partition_key.to_string())
        }
    }

    if removed_partitions.len() > 0 {
        if let Some(attr) = attr {
            app.dispatch_event(TransactionEvent::DeletePartitions {
                table: db_table.into(),
                partitions: removed_partitions,
                attr,
            })
            .await
        }
    }
}

pub async fn bulk_insert_or_update(
    app: &AppServices,
    db_table: &DbTable,
    payload: &[u8],
    attr: Option<TransactionAttributes>,
) -> Result<(), DbOperationFail> {
    let db_rows_by_partition =
        crate::db_entity::json_parser::parse_db_rows_to_hash_map_by_partition_key(payload)?;

    bulk_insert_or_update_execute(app, db_table, db_rows_by_partition, attr).await;

    Ok(())
}

pub async fn bulk_insert_or_update_execute(
    app: &AppServices,
    db_table: &DbTable,
    mut rows_by_partition: HashMap<String, Vec<DbRow>>,
    attr: Option<TransactionAttributes>,
) {
    let now = MyDateTime::utc_now();

    let mut table_write_access = db_table.data.write().await;

    let mut sync = HashMap::new();

    for (partition_key, mut db_rows) in rows_by_partition.drain() {
        let db_partition = table_write_access
            .get_or_create_partition_and_update_last_access(partition_key.as_str(), now);

        for db_row in db_rows.drain(..) {
            let db_row = Arc::new(db_row);

            db_partition
                .rows
                .insert(db_row.row_key.to_string(), db_row.clone());

            if attr.is_some() {
                if !sync.contains_key(partition_key.as_str()) {
                    sync.insert(partition_key.to_string(), Vec::new());
                }
                sync.get_mut(partition_key.as_str()).unwrap().push(db_row);
            }
        }
    }

    if let Some(attr) = attr {
        app.dispatch_event(TransactionEvent::update_rows(db_table, attr, sync))
            .await
    }
}

pub async fn clean_table_and_bulk_insert(
    app: &AppServices,
    db_table: &DbTable,
    payload: &[u8],
    attr: Option<TransactionAttributes>,
) -> Result<(), DbOperationFail> {
    let now = MyDateTime::utc_now();
    let entities =
        crate::db_entity::json_parser::parse_entities_to_hash_map_by_partition_key(payload)?;

    let mut write_access = db_table.data.write().await;

    let mut sync = HashMap::new();

    if write_access.clear() {
        if let Some(attr_ref) = &attr {
            let evnt = TransactionEvent::CleanTable {
                table: db_table.into(),
                attr: attr_ref.clone(),
            };
            app.dispatch_event(evnt).await;
        }
    }

    for (partition_key, rows) in entities {
        let db_partition = write_access
            .get_or_create_partition_and_update_last_access(partition_key.as_str(), now);

        let mut rows_to_sync = Vec::new();

        for db_entity in &rows {
            let db_row = Arc::new(DbRow::form_db_entity(db_entity));
            db_partition
                .rows
                .insert(db_row.row_key.to_string(), db_row.clone());
            rows_to_sync.push(db_row);
        }

        sync.insert(partition_key.to_string(), rows_to_sync);
    }

    if let Some(attr) = attr {
        app.dispatch_event(TransactionEvent::update_rows(db_table, attr, sync))
            .await;
    }

    Ok(())
}

pub async fn clean_partition_and_bulk_insert(
    app: &AppServices,
    db_table: &DbTable,
    partition_key: &str,
    payload: &[u8],
    attr: Option<TransactionAttributes>,
) -> Result<(), DbOperationFail> {
    let entities =
        crate::db_entity::json_parser::parse_entities_to_hash_map_by_partition_key(payload)?;

    let now = MyDateTime::utc_now();

    let mut write_access = db_table.data.write().await;

    let mut sync = HashMap::new();

    if write_access.clear_partition(partition_key) {
        if let Some(attr_ref) = &attr {
            let evnt = TransactionEvent::CleanTable {
                table: db_table.into(),
                attr: attr_ref.clone(),
            };
            app.dispatch_event(evnt).await;
        }
    }

    for (partition_key, rows) in entities {
        let db_partition = write_access
            .get_or_create_partition_and_update_last_access(partition_key.as_str(), now);

        let mut rows_to_sync = Vec::new();

        for db_entity in &rows {
            let db_row = Arc::new(DbRow::form_db_entity(db_entity));
            db_partition
                .rows
                .insert(db_row.row_key.to_string(), db_row.clone());
            rows_to_sync.push(db_row);
        }

        sync.insert(partition_key.to_string(), rows_to_sync);
    }

    if let Some(attr) = attr {
        app.dispatch_event(TransactionEvent::update_rows(db_table, attr, sync))
            .await;
    }

    Ok(())
}

pub async fn bulk_delete(
    app: &AppServices,
    db_table: &DbTable,
    payload: &[u8],
    attr: Option<TransactionAttributes>,
) {
    let rows_to_delete: HashMap<String, Vec<String>> = serde_json::from_slice(payload).unwrap();

    let mut write_access = db_table.data.write().await;

    let mut sync = HashMap::new();

    for (partition_key, row_keys) in &rows_to_delete {
        let partition = write_access.get_partition_mut(partition_key);

        if partition.is_none() {
            continue;
        }

        let partition = partition.unwrap();

        let mut deleted_rows = Vec::new();

        for row_key in row_keys {
            let removed = partition.rows.remove(row_key);

            if removed.is_none() {
                continue;
            }

            let removed = removed.unwrap();
            deleted_rows.push(removed.row_key.to_string());
        }

        if partition.rows.len() == 0 {
            write_access.remove_partition(partition_key);
        }

        if deleted_rows.len() > 0 {
            sync.insert(partition_key.to_string(), deleted_rows);
        }
    }

    if let Some(attr) = attr {
        app.dispatch_event(TransactionEvent::DeleteRows {
            table: db_table.into(),
            attr,
            rows: sync,
        })
        .await;
    }
}

#[derive(Deserialize, Serialize)]
pub struct DeleteModel {
    pub key: String,
    pub values: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let str = r###"{"Key1": ["Value1", "Value2"], "Key2": ["Value3", "Value4"]}"###;

        let result: HashMap<String, Vec<String>> = serde_json::from_slice(str.as_bytes()).unwrap();

        assert_eq!(2, result.len())
    }
}
