use std::{collections::HashMap, sync::Arc};

use crate::{
    app::AppServices,
    date_time::MyDateTime,
    db::{DbOperationFail, DbTable, DbTableAttributes, DbTableData},
    db_transactions::{TransactionAttributes, TransactionEvent},
};

fn create_table_with_write_access(
    tables_write_access: &mut HashMap<String, Arc<DbTable>>,
    name: &str,
    persist: bool,
    max_partitions_amount: Option<usize>,
) -> Result<Arc<DbTable>, DbOperationFail> {
    if tables_write_access.contains_key(name) {
        return Err(DbOperationFail::TableAlreadyExist {
            table_name: name.to_string(),
        });
    }

    let table_attributes = DbTableAttributes {
        persist,
        max_partitions_amount,
    };

    let db_table_data = DbTableData::new(table_attributes);

    let new_table = DbTable::new(name.to_string(), db_table_data, MyDateTime::utc_now());

    let new_table = Arc::new(new_table);
    tables_write_access.insert(name.to_string(), new_table.clone());

    return Ok(new_table);
}

pub async fn create_table(
    app: &AppServices,
    table_name: &str,
    persist_table: bool,
    max_partitions_amount: Option<usize>,
    attr: Option<TransactionAttributes>,
) -> Result<Arc<DbTable>, DbOperationFail> {
    let mut tables_write_access = app.db.tables.write().await;

    let db_table = create_table_with_write_access(
        &mut tables_write_access,
        table_name,
        persist_table,
        max_partitions_amount,
    )?;

    if let Some(attr) = attr {
        let table_read_access = db_table.data.read().await;
        app.dispatch_event(TransactionEvent::init_table(
            db_table.clone(),
            attr,
            table_read_access.get_snapshot(),
        ))
        .await;
    }

    Ok(db_table)
}

async fn get_or_create_table(
    app: &AppServices,
    table_name: &str,
    persist_table: bool,
    max_partitions_amount: Option<usize>,
    attr: &Option<TransactionAttributes>,
) -> Arc<DbTable> {
    let mut tables_write_access = app.db.tables.write().await;

    let db_table_result = tables_write_access.get(table_name);

    if let Some(db_table) = db_table_result {
        return db_table.clone();
    }

    let db_table = create_table_with_write_access(
        &mut tables_write_access,
        table_name,
        persist_table,
        max_partitions_amount,
    )
    .unwrap();

    if let Some(attr) = attr {
        let table_read_access = db_table.data.read().await;
        app.dispatch_event(TransactionEvent::init_table(
            db_table.clone(),
            attr.clone(),
            table_read_access.get_snapshot(),
        ))
        .await;
    }

    db_table
}

pub async fn create_table_if_not_exist(
    app: &AppServices,
    table_name: &str,
    persist_table: bool,
    max_partitions_amount: Option<usize>,
    attr: Option<TransactionAttributes>,
) -> Arc<DbTable> {
    let db_table =
        get_or_create_table(app, table_name, persist_table, max_partitions_amount, &attr).await;

    set_table_attrubutes(
        app,
        db_table.clone(),
        true,
        persist_table,
        max_partitions_amount,
        attr,
    )
    .await;

    db_table
}

pub async fn set_table_attrubutes(
    app: &AppServices,
    db_table: Arc<DbTable>,
    table_is_just_created: bool,
    persist: bool,
    max_partitions_amount: Option<usize>,
    attr: Option<TransactionAttributes>,
) {
    let mut table_write_access = db_table.data.write().await;

    let result = table_write_access.set_table_attributes(persist, max_partitions_amount);

    if result {
        if let Some(attr) = attr {
            app.dispatch_event(TransactionEvent::UpdateTableAttributes {
                table: db_table.clone(),
                attr,
                table_is_just_created,
                persist,
                max_partitions_amount,
            })
            .await;
        }
    }
}

pub async fn delete_table(
    app: &AppServices,
    table_name: &str,
    attr: Option<TransactionAttributes>,
) -> Result<(), DbOperationFail> {
    match app.db.delete_table(table_name).await {
        Some(db_table) => {
            if let Some(attr) = attr {
                app.dispatch_event(TransactionEvent::DeleteTable {
                    table: db_table.clone(),
                    attr,
                })
                .await;
            }

            Ok(())
        }
        None => Err(DbOperationFail::TableNotFound {
            table_name: table_name.to_string(),
        }),
    }
}
