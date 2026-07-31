use std::sync::Arc;

use my_no_sql_sdk::core::db::{DbTableAttributes, DbTableInner};
use my_no_sql_sdk::core::rust_extensions::date_time::DateTimeAsMicroseconds;
use my_no_sql_sdk::server::DbTable;

use crate::{
    app::{AppContext, DbNamespace},
    data_readers::DataReader,
    db_operations::DbOperationError,
    db_sync::{states::TableFirstInitSyncData, SyncEvent},
};

pub async fn subscribe(
    app: &Arc<AppContext>,
    db_namespace: &Arc<DbNamespace>,
    data_reader: Arc<DataReader>,
    table_name: &str,
) -> Result<(), DbOperationError> {
    // The reader is routed by the namespace recorded on it, so it has to be
    // recorded here rather than by whoever happened to open the connection: a
    // TCP reader gets it from the SetNamespace packet, but an HTTP reader names
    // its namespace on the subscribe request itself and would otherwise stay
    // labelled `default` — receiving the default namespace's updates and none of
    // its own.
    if let Err(err) = data_reader.set_namespace(db_namespace.name.clone()).await {
        return Err(DbOperationError::NamespaceNameValidationError(err));
    }

    let mut table = db_namespace.db.get_table(table_name);

    if table.is_none() {
        if app.settings.auto_create_table_on_reader_subscribe {
            println!(
                "Table {} does not exist. Creating it now on reader {:?} subscribe",
                table_name,
                data_reader.get_name()
            );

            table = crate::db_operations::write::table::create_if_not_exist(
                app,
                db_namespace,
                table_name,
                false,
                None,
                None,
                crate::db_sync::EventSource::Subscriber,
                DateTimeAsMicroseconds::now(),
            )
            .await?
            .into();
        } else {
            println!(
                "{:?} is subscribing to the table {} which does not exist. \
                 Sending an empty snapshot so the reader gets initialized",
                data_reader.get_name(),
                table_name
            );

            // Table does not exist and auto-create is disabled. Build a throwaway,
            // empty DbTable - it is NOT inserted into app.db and NO subscription is
            // registered. It exists only to produce an empty InitTable snapshot so the
            // reader initializes instead of receiving an Error contract (which makes
            // the SDK reader panic).
            let empty_table = DbTable::new(DbTableInner::new(
                table_name.into(),
                DbTableAttributes::new(false, None, None, false, DateTimeAsMicroseconds::now()),
            ));

            crate::operations::sync::dispatch(
                app,
                db_namespace,
                SyncEvent::TableFirstInit(TableFirstInitSyncData {
                    db_table: empty_table,
                    data_reader,
                }),
            );

            return Ok(());
        }
    }

    let db_table = table.unwrap();

    data_reader.subscribe(&db_table).await;

    crate::operations::sync::dispatch(
        app,
        db_namespace,
        SyncEvent::TableFirstInit(TableFirstInitSyncData {
            db_table,
            data_reader,
        }),
    );

    Ok(())
}
