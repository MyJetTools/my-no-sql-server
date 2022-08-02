use std::sync::Arc;

use my_no_sql_core::db::{DbTable, DbTableAttributes};
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::db::DbTableWrapper;
use crate::db_operations::validation;
use crate::{
    app::AppContext,
    db_operations::DbOperationError,
    db_sync::{states::InitTableEventSyncData, EventSource, SyncEvent},
};

pub async fn create(
    app: &AppContext,
    table_name: &str,
    persist_table: bool,
    max_partitions_amount: Option<usize>,
    event_src: EventSource,
) -> Result<Arc<DbTableWrapper>, DbOperationError> {
    super::super::check_app_states(app)?;

    validation::validate_table_name(table_name)?;

    let now = DateTimeAsMicroseconds::now();

    let create_table_result =
        get_or_create_table(app, table_name, persist_table, max_partitions_amount, now).await;

    match create_table_result {
        CreateTableResult::JustCreated(db_table_wrapper) => {
            {
                let mut write_access = db_table_wrapper.data.write().await;

                write_access
                    .persist_markers
                    .data_to_persist
                    .mark_persist_attrs();
            }

            let state = InitTableEventSyncData::new(db_table_wrapper.clone(), event_src);

            crate::operations::sync::dispatch(app, SyncEvent::InitTable(state));

            return Ok(db_table_wrapper);
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
    event_src: EventSource,
) -> Result<Arc<DbTableWrapper>, DbOperationError> {
    validation::validate_table_name(table_name)?;
    let now = DateTimeAsMicroseconds::now();

    let create_table_result =
        get_or_create_table(app, table_name, persist_table, max_partitions_amount, now).await;

    match create_table_result {
        CreateTableResult::JustCreated(db_table_wrapper) => {
            let state = InitTableEventSyncData::new(db_table_wrapper.clone(), event_src);

            crate::operations::sync::dispatch(app, SyncEvent::InitTable(state));

            {
                let mut write_access = db_table_wrapper.data.write().await;

                write_access
                    .persist_markers
                    .data_to_persist
                    .mark_persist_attrs();
            }

            return Ok(db_table_wrapper);
        }
        CreateTableResult::AlreadyHadTable(db_table) => {
            return Ok(db_table);
        }
    }
}

pub async fn create_if_not_exist(
    app: &Arc<AppContext>,
    table_name: &str,
    persist_table: bool,
    max_partitions_amount: Option<usize>,
    event_src: EventSource,
) -> Result<Arc<DbTableWrapper>, DbOperationError> {
    super::super::check_app_states(app)?;
    validation::validate_table_name(table_name)?;

    let db_table_wrapper = get_or_create(
        app,
        table_name,
        persist_table,
        max_partitions_amount,
        event_src.clone(),
    )
    .await?;

    set_table_attrubutes(
        app,
        db_table_wrapper.clone(),
        persist_table,
        max_partitions_amount,
    )
    .await?;

    Ok(db_table_wrapper)
}

pub async fn update_persist_state(
    app: &Arc<AppContext>,
    db_table_wrapper: Arc<DbTableWrapper>,
    persist: bool,
) -> Result<(), DbOperationError> {
    super::super::check_app_states(app)?;

    let mut write_access = db_table_wrapper.data.write().await;

    if write_access.db_table.attributes.persist == persist {
        return Ok(());
    }

    write_access.db_table.attributes.persist = persist;

    write_access
        .persist_markers
        .data_to_persist
        .mark_persist_attrs();

    Ok(())
}

pub async fn set_table_attrubutes(
    app: &Arc<AppContext>,
    db_table_wrapper: Arc<DbTableWrapper>,

    persist: bool,
    max_partitions_amount: Option<usize>,
) -> Result<(), DbOperationError> {
    super::super::check_app_states(app)?;

    let mut write_access = db_table_wrapper.data.write().await;
    let result = write_access
        .db_table
        .attributes
        .update(persist, max_partitions_amount);

    if !result {
        return Ok(());
    }

    write_access
        .persist_markers
        .data_to_persist
        .mark_persist_attrs();

    Ok(())
}

pub async fn delete(
    _app: Arc<AppContext>,
    _table_name: String,
    _event_src: EventSource,
    _persist_moment: DateTimeAsMicroseconds,
) -> Result<(), DbOperationError> {
    todo!("Delete Temporary deleted");
    /*
    super::super::check_app_states(app.as_ref())?;
    let result = app.db.delete_table(table_name.as_str()).await;

    if result.is_none() {
        return Err(DbOperationError::TableNotFound(table_name.to_string()));
    }

    let db_table = result.unwrap();

    {
        let db_table = db_table.data.read().await;

        //TODO - Fix Delete Table Case

        app.persist_markers
            .persist_table(db_table.name.as_str(), persist_moment)
            .await;

        crate::operations::sync::dispatch(
            app.as_ref(),
            SyncEvent::DeleteTable(DeleteTableSyncData::new(&db_table, event_src)),
        );
    }

    Ok(())
     */
}

pub async fn init(app: &AppContext, db_table: DbTable) {
    let db_table = DbTableWrapper::new(db_table);
    let mut tables_write_access = app.db.tables.write().await;
    tables_write_access.insert(db_table.name.to_string(), db_table);
}

enum CreateTableResult {
    JustCreated(Arc<DbTableWrapper>),
    AlreadyHadTable(Arc<DbTableWrapper>),
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

    let attributes = DbTableAttributes {
        persist,
        max_partitions_amount,
        created: now,
    };

    let new_table = DbTableWrapper::new(DbTable::new(table_name.to_string(), attributes));

    write_access.insert(table_name.to_string(), new_table.clone());

    return CreateTableResult::JustCreated(new_table);
}
