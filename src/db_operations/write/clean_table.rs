use std::sync::Arc;

use crate::{
    app::AppContext,
    db::{DbTable, DbTableSnapshot},
    db_sync::{states::InitTableEventSyncData, SyncAttributes, SyncEvent},
};

pub async fn execute(app: &AppContext, db_table: Arc<DbTable>, attr: Option<SyncAttributes>) {
    let mut table_write_access = db_table.data.write().await;

    if table_write_access.partitions.len() == 0 {
        return;
    }

    let removed = super::db_actions::clean_table(app, &mut table_write_access).await;

    if removed.is_some() {
        if let Some(attr) = attr {
            let mut init_state = InitTableEventSyncData::new(db_table.as_ref(), attr);

            init_state.add_table_snapshot(DbTableSnapshot::new(&table_write_access));

            app.events_dispatcher
                .dispatch(SyncEvent::InitTable(init_state))
                .await
        }
    }
}
