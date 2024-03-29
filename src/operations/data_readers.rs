use std::sync::Arc;

use my_no_sql_sdk::core::rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    app::AppContext,
    data_readers::DataReader,
    db_operations::DbOperationError,
    db_sync::{states::TableFirstInitSyncData, SyncEvent},
};

pub async fn subscribe(
    app: &Arc<AppContext>,
    data_reader: Arc<DataReader>,
    table_name: &str,
) -> Result<(), DbOperationError> {
    let mut table = app.db.get_table(table_name).await;

    if table.is_none() {
        if app.settings.auto_create_table_on_reader_subscribe {
            println!(
                "Table {} does not exist. Creating it now on reader {:?} subscribe",
                table_name,
                data_reader.get_name()
            );

            table = crate::db_operations::write::table::create_if_not_exist(
                app,
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
                "{:?} is subscribing to the table {} which does not exist",
                data_reader.get_name(),
                table_name
            );

            return Err(DbOperationError::TableNotFound(table_name.to_string()));
        }
    }

    let db_table = table.unwrap();

    data_reader.subscribe(&db_table).await;

    crate::operations::sync::dispatch(
        app,
        SyncEvent::TableFirstInit(TableFirstInitSyncData {
            db_table,
            data_reader,
        }),
    );

    Ok(())
}
