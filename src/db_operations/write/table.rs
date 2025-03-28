use std::sync::Arc;

use my_no_sql_sdk::core::db::{DbTableAttributes, DbTableInner};
use my_no_sql_sdk::core::rust_extensions::date_time::DateTimeAsMicroseconds;
use my_no_sql_sdk::server::DbTable;

use crate::db_operations::validation;
use crate::{
    app::AppContext,
    db_operations::DbOperationError,
    db_sync::{
        states::{
            DeleteTableSyncData, InitTableEventSyncData, SyncTableData,
            UpdateTableAttributesSyncData,
        },
        EventSource, SyncEvent,
    },
};

pub async fn create(
    app: &AppContext,
    table_name: &str,
    persist_table: bool,
    max_partitions_amount: Option<usize>,
    max_rows_per_partition_amount: Option<usize>,
    event_src: EventSource,
    persist_moment: DateTimeAsMicroseconds,
) -> Result<Arc<DbTable>, DbOperationError> {
    super::super::check_app_states(app)?;

    validation::validate_table_name(table_name)?;

    let now = DateTimeAsMicroseconds::now();

    let create_table_result = get_or_create_table(
        app,
        table_name,
        persist_table,
        max_partitions_amount,
        max_rows_per_partition_amount,
        now,
    )
    .await;

    match create_table_result {
        CreateTableResult::JustCreated(db_table) => {
            {
                let table_data = db_table.data.write().await;

                app.persist_markers
                    .persist_table_attributes(&table_data.name, persist_moment)
                    .await;

                app.persist_markers
                    .persist_table_attributes(&table_data.name, persist_moment)
                    .await;

                let state = InitTableEventSyncData::new(&table_data, event_src);

                crate::operations::sync::dispatch(app, SyncEvent::InitTable(state));
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
    max_rows_per_partition_amount: Option<usize>,
    event_src: EventSource,
    persist_moment: DateTimeAsMicroseconds,
) -> Result<Arc<DbTable>, DbOperationError> {
    validation::validate_table_name(table_name)?;
    let now = DateTimeAsMicroseconds::now();

    let create_table_result = get_or_create_table(
        app,
        table_name,
        persist_table,
        max_partitions_amount,
        max_rows_per_partition_amount,
        now,
    )
    .await;

    match create_table_result {
        CreateTableResult::JustCreated(db_table) => {
            {
                let table_data = db_table.data.write().await;
                let state = InitTableEventSyncData::new(&table_data, event_src);

                crate::operations::sync::dispatch(app, SyncEvent::InitTable(state));

                app.persist_markers
                    .persist_table_attributes(&table_data.name, persist_moment)
                    .await;
            }

            return Ok(db_table);
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
    max_rows_per_partition_amount: Option<usize>,
    event_src: EventSource,

    persist_moment: DateTimeAsMicroseconds,
) -> Result<Arc<DbTable>, DbOperationError> {
    super::super::check_app_states(app)?;

    validation::validate_table_name(table_name)?;

    let db_table = get_or_create(
        app,
        table_name,
        persist_table,
        max_partitions_amount,
        max_rows_per_partition_amount,
        event_src.clone(),
        persist_moment,
    )
    .await?;

    set_table_attributes(
        app,
        db_table.clone(),
        persist_table,
        max_partitions_amount,
        max_rows_per_partition_amount,
        event_src,
    )
    .await?;

    Ok(db_table)
}

pub async fn update_persist_state(
    app: &Arc<AppContext>,
    db_table: Arc<DbTable>,
    persist: bool,
    event_src: EventSource,
) -> Result<(), DbOperationError> {
    super::super::check_app_states(app)?;
    let attrs = db_table.get_attributes().await;

    set_table_attributes(
        app,
        db_table,
        persist,
        attrs.max_partitions_amount,
        attrs.max_rows_per_partition_amount,
        event_src,
    )
    .await?;
    Ok(())
}

pub async fn set_table_attributes(
    app: &Arc<AppContext>,
    db_table: Arc<DbTable>,
    persist: bool,
    max_partitions_amount: Option<usize>,
    max_rows_per_partition_amount: Option<usize>,
    event_src: EventSource,
) -> Result<(), DbOperationError> {
    super::super::check_app_states(app)?;

    let mut write_access = db_table.data.write().await;
    let result = write_access.attributes.update(
        persist,
        max_partitions_amount,
        max_rows_per_partition_amount,
    );

    if !result {
        return Ok(());
    }

    crate::operations::sync::dispatch(
        app,
        SyncEvent::UpdateTableAttributes(UpdateTableAttributesSyncData {
            table_data: SyncTableData {
                table_name: db_table.name.clone(),
            },
            event_src,
        }),
    );

    app.persist_markers
        .persist_table_attributes(&write_access.name, DateTimeAsMicroseconds::now())
        .await;

    Ok(())
}

pub async fn delete(
    app: Arc<AppContext>,
    table_name: String,
    event_src: EventSource,
    persist_moment: DateTimeAsMicroseconds,
) -> Result<(), DbOperationError> {
    super::super::check_app_states(app.as_ref())?;
    let result = app.db.delete_table(table_name.as_str()).await;

    if result.is_none() {
        return Err(DbOperationError::TableNotFound(table_name));
    }

    let db_table = result.unwrap();

    {
        let table_data = db_table.data.read().await;

        app.persist_markers
            .persist_table_attributes(&table_data.name, persist_moment)
            .await;

        crate::operations::sync::dispatch(
            app.as_ref(),
            SyncEvent::DeleteTable(DeleteTableSyncData::new(&table_data, event_src)),
        );
    }

    let app = app.clone();
    let table_name = db_table.name.clone();
    tokio::spawn(async move {
        crate::operations::persist::scripts::delete_table(&app, &table_name).await
    });

    Ok(())
}

pub async fn init(app: &AppContext, db_table: DbTableInner) -> Arc<DbTable> {
    let db_table = DbTable::new(db_table);
    let mut tables_write_access = app.db.tables.write().await;

    tables_write_access.insert(db_table.name.to_string(), db_table.clone());

    db_table
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
    max_rows_per_partition_amount: Option<usize>,
    now: DateTimeAsMicroseconds,
) -> CreateTableResult {
    let mut write_access = app.db.tables.write().await;

    if let Some(table) = write_access.get(table_name) {
        return CreateTableResult::AlreadyHadTable(table.clone());
    }

    let table_attributes = DbTableAttributes {
        persist,
        max_partitions_amount,
        created: now,
        max_rows_per_partition_amount,
    };

    let db_table = DbTableInner::new(table_name.into(), table_attributes);

    let db_table_wrapper = DbTable::new(db_table);

    write_access.insert(table_name.to_string(), db_table_wrapper.clone());

    return CreateTableResult::JustCreated(db_table_wrapper);
}
