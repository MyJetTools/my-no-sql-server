use std::sync::Arc;

use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    app::AppContext,
    db::{DbTable, DbTableData},
    db_operations::DbOperationError,
    db_sync::{
        states::{
            DeleteTableSyncData, InitTableEventSyncData, SyncTableData,
            UpdateTableAttributesSyncData,
        },
        SyncAttributes, SyncEvent,
    },
};

pub async fn create(
    app: &AppContext,
    table_name: &str,
    persist_table: bool,
    max_partitions_amount: Option<usize>,
    attr: Option<SyncAttributes>,
) -> Result<Arc<DbTable>, DbOperationError> {
    let now = DateTimeAsMicroseconds::now();

    let create_table_result =
        get_or_create_table(app, table_name, persist_table, max_partitions_amount, now).await;

    match create_table_result {
        CreateTableResult::JustCreated(db_table) => {
            if let Some(attr) = attr {
                let table_data = db_table.data.read().await;
                let state = InitTableEventSyncData::new(&table_data, attr);
                app.events_dispatcher
                    .dispatch(SyncEvent::InitTable(state))
                    .await;
            }

            return Ok(db_table);
        }
        CreateTableResult::AlreadyHadTable(_) => {
            return Err(DbOperationError::TableAlreadyExists);
        }
    }
}

async fn get_or_create(
    app: &AppContext,
    table_name: &str,
    persist_table: bool,
    max_partitions_amount: Option<usize>,
    attr: Option<SyncAttributes>,
) -> Arc<DbTable> {
    let now = DateTimeAsMicroseconds::now();

    let create_table_result =
        get_or_create_table(app, table_name, persist_table, max_partitions_amount, now).await;

    match create_table_result {
        CreateTableResult::JustCreated(db_table) => {
            if let Some(attr) = attr {
                let table_data = db_table.data.read().await;
                let state = InitTableEventSyncData::new(&table_data, attr);
                app.events_dispatcher
                    .dispatch(SyncEvent::InitTable(state))
                    .await;
            }

            return db_table;
        }
        CreateTableResult::AlreadyHadTable(db_table) => {
            return db_table;
        }
    }
}

pub async fn create_if_not_exist(
    app: &AppContext,
    table_name: &str,
    persist_table: bool,
    max_partitions_amount: Option<usize>,
    attr: Option<SyncAttributes>,
) -> Arc<DbTable> {
    let db_table = get_or_create(
        app,
        table_name,
        persist_table,
        max_partitions_amount,
        attr.clone(),
    )
    .await;

    set_table_attrubutes(
        app,
        db_table.clone(),
        persist_table,
        max_partitions_amount,
        attr,
    )
    .await;

    db_table
}

pub async fn set_table_attrubutes(
    app: &AppContext,
    db_table: Arc<DbTable>,

    persist: bool,
    max_partitions_amount: Option<usize>,
    attr: Option<SyncAttributes>,
) {
    let result = db_table
        .set_table_attributes(persist, max_partitions_amount)
        .await;

    if result {
        if let Some(attr) = attr {
            app.events_dispatcher
                .dispatch(SyncEvent::UpdateTableAttributes(
                    UpdateTableAttributesSyncData {
                        table_data: SyncTableData {
                            table_name: db_table.name.to_string(),
                            persist,
                        },
                        attr,
                        persist,
                        max_partitions_amount,
                    },
                ))
                .await;
        }
    }
}

pub async fn delete(
    app: &AppContext,
    table_name: &str,
    attr: Option<SyncAttributes>,
) -> Result<(), DbOperationError> {
    let result = app.db.delete_table(table_name).await;

    if result.is_none() {
        return Err(DbOperationError::TableNotFound(table_name.to_string()));
    }

    let db_table = result.unwrap();

    if let Some(attr) = attr {
        let table_data = db_table.data.read().await;
        app.events_dispatcher
            .dispatch(SyncEvent::DeleteTable(DeleteTableSyncData::new(
                &table_data,
                attr,
            )))
            .await;
    }

    Ok(())
}

pub async fn init(app: &AppContext, table_data: DbTableData, now: DateTimeAsMicroseconds) {
    app.blob_content_cache.init(&table_data).await;

    let db_table = DbTable::new(table_data, now);
    let mut tables_write_access = app.db.tables.write().await;

    tables_write_access.insert(db_table.name.to_string(), Arc::new(db_table));
}

enum CreateTableResult {
    JustCreated(Arc<DbTable>),
    AlreadyHadTable(Arc<DbTable>),
}

async fn get_or_create_table(
    app: &AppContext,
    table_name: &str,
    persist: bool,
    max_partitions_amount: Option<usize>,
    now: DateTimeAsMicroseconds,
) -> CreateTableResult {
    let mut write_access = app.db.tables.write().await;

    if let Some(table) = write_access.get(table_name) {
        return CreateTableResult::AlreadyHadTable(table.clone());
    }

    let table_attributes = crate::db::DbTableAttributes {
        persist,
        max_partitions_amount,
        created: now,
    };

    let db_table_data = DbTableData::new(table_name.to_string(), table_attributes);

    let new_table = DbTable::new(db_table_data, DateTimeAsMicroseconds::now());

    let new_table = Arc::new(new_table);
    write_access.insert(table_name.to_string(), new_table.clone());

    return CreateTableResult::JustCreated(new_table);
}
