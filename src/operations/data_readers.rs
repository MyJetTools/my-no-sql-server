use std::sync::Arc;

use crate::{
    app::AppContext,
    data_readers::DataReader,
    db_sync::{states::TableFirstInitSyncData, SyncEvent},
    operations::OperationError,
};

pub async fn subscribe(
    app: &AppContext,
    data_reader: Arc<DataReader>,
    table_name: &str,
) -> Result<(), OperationError> {
    let table = app.db.get_table(table_name).await;

    if table.is_none() {
        println!(
            "{:?} is subscribing to the table {} which does not exist",
            data_reader.get_name().await,
            table_name
        );

        return Err(OperationError::TableNotFound);
    }

    let db_table = table.unwrap();

    data_reader.subscribe(db_table.clone()).await;

    app.events_dispatcher
        .dispatch(SyncEvent::TableFirstInit(TableFirstInitSyncData {
            db_table,
            data_reader,
        }));

    Ok(())
}
