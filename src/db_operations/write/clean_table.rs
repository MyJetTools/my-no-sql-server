use std::{collections::BTreeMap, sync::Arc};

use crate::{
    app::AppContext,
    db::{DbTable, DbTableData},
    db_sync::{states::InitTableEventState, SyncAttributes, SyncEvent},
};

pub async fn execute(app: &AppContext, db_table: Arc<DbTable>, attr: Option<SyncAttributes>) {
    let mut table_write_access = db_table.data.write().await;

    if table_write_access.partitions.len() == 0 {
        return;
    }

    let sync = if let Some(attr) = attr {
        let mut init_state = InitTableEventState::new(db_table.clone(), attr);
        clean_table(&mut table_write_access, &mut init_state);

        app.events_dispatcher
            .dispatch(SyncEvent::InitTable(init_state))
            .await
    } else {
        table_write_access.partitions.clear();
    };
}

pub fn clean_table(db_table_data: &mut DbTableData, init_state: &mut InitTableEventState) {
    let mut old_partitions = BTreeMap::new();
    std::mem::swap(&mut old_partitions, &mut db_table_data.partitions);

    for (partition_key, db_partition) in old_partitions {
        init_state.add_cleaned_partition_before(partition_key, db_partition);
    }
}
