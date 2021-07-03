use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

use crate::{
    app::AppServices,
    date_time::MyDateTime,
    db::{DbRow, DbTable, FailOperationResult, OperationResult},
    db_transactions::{TransactionAttributes, TransactionEvent},
    json::{array_parser, db_entity::DbEntity},
};

pub async fn insert(
    app: &AppServices,
    db_table: &DbTable,
    payload: &[u8],
    attr: Option<TransactionAttributes>,
) -> Result<OperationResult, FailOperationResult> {
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

    return Ok(OperationResult::Ok);
}

pub async fn insert_or_replace(
    app: &AppServices,
    db_table: &DbTable,
    payload: &[u8],
    attr: Option<TransactionAttributes>,
) -> Result<OperationResult, FailOperationResult> {
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

    return Ok(OperationResult::Ok);
}

pub async fn replace(
    app: &AppServices,
    db_table: &DbTable,
    payload: &[u8],
    attr: Option<TransactionAttributes>,
) -> Result<OperationResult, FailOperationResult> {
    let entity = DbEntity::parse(payload)?;

    if entity.time_stamp.is_none() {
        return Err(FailOperationResult::TimeStampFieldRequires);
    }

    let entity_time_stamp = entity.time_stamp.unwrap();

    let mut write_access = db_table.data.write().await;

    let now = MyDateTime::utc_now();

    let db_partition =
        write_access.get_partition_and_update_last_access_mut(entity.partition_key.as_str(), now);

    if db_partition.is_none() {
        return Err(FailOperationResult::RecordNotFound);
    }

    let db_partition = db_partition.unwrap();

    let db_row = db_partition.get_row_and_update_last_time(entity.row_key.as_str(), now);

    if db_row.is_none() {
        return Err(FailOperationResult::RecordNotFound);
    }

    let db_row = db_row.unwrap();

    if !db_row.time_stamp.equals_to(entity_time_stamp) {
        return Err(FailOperationResult::OptimisticConcurencyUpdateFails);
    }

    let db_row = Arc::new(DbRow::form_db_entity(&entity));

    db_partition.insert_or_replace(db_row.clone());

    if let Some(attr) = attr {
        app.dispatch_event(TransactionEvent::UpdateRow {
            table_name: db_table.name.clone(),
            attr,
            partition_key: entity.partition_key.to_string(),
            row: db_row,
        })
        .await
    }

    Ok(OperationResult::Ok)
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
                table_name: db_table.name.clone(),
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
) -> Result<(), FailOperationResult> {
    let entities = array_parser::split_to_objects(payload)?;
    let now = MyDateTime::utc_now();

    let mut table_write_access = db_table.data.write().await;

    let mut sync = HashMap::new();

    for db_entity in &entities {
        let db_partition = table_write_access
            .get_or_create_partition_and_update_last_access(db_entity.partition_key.as_str(), now);

        let db_row = DbRow::form_db_entity(db_entity);

        let db_row = Arc::new(db_row);

        db_partition
            .rows
            .insert(db_entity.row_key.to_string(), db_row.clone());

        if attr.is_some() {
            if !sync.contains_key(db_entity.partition_key.as_str()) {
                sync.insert(db_entity.partition_key.to_string(), Vec::new());
            }
            sync.get_mut(db_entity.partition_key.as_str())
                .unwrap()
                .push(db_row);
        }
    }

    if let Some(attr) = attr {
        app.dispatch_event(TransactionEvent::update_rows(db_table, attr, sync))
            .await
    }

    Ok(())
}

pub async fn clean_table_and_bulk_insert(
    app: &AppServices,
    db_table: &DbTable,
    payload: &[u8],
    attr: Option<TransactionAttributes>,
) -> Result<(), FailOperationResult> {
    let entities = array_parser::split_to_objects(payload)?;
    let now = MyDateTime::utc_now();
    let entities = to_hash_map(entities);

    let mut write_access = db_table.data.write().await;

    let mut sync = HashMap::new();

    if write_access.clear() {
        if let Some(attr_ref) = &attr {
            let evnt = TransactionEvent::CleanTable {
                table_name: db_table.name.to_string(),
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
) -> Result<(), FailOperationResult> {
    let entities = array_parser::split_to_objects(payload)?;

    let now = MyDateTime::utc_now();

    let entities = to_hash_map(entities);

    let mut write_access = db_table.data.write().await;

    let mut sync = HashMap::new();

    if write_access.clear_partition(partition_key) {
        if let Some(attr_ref) = &attr {
            let evnt = TransactionEvent::CleanTable {
                table_name: db_table.name.to_string(),
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
) -> Result<(), FailOperationResult> {
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
            deleted_rows.push(removed);
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
            table_name: db_table.name.to_string(),
            attr,
            rows: sync,
        })
        .await;
    }

    Ok(())
}

fn to_hash_map(mut src: Vec<DbEntity>) -> HashMap<String, Vec<DbEntity>> {
    let mut result = HashMap::new();

    for entity in src.drain(..) {
        if !result.contains_key(entity.partition_key.as_str()) {
            result.insert(entity.partition_key.to_string(), Vec::new());

            result
                .get_mut(entity.partition_key.as_str())
                .unwrap()
                .push(entity)
        }
    }
    return result;
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
