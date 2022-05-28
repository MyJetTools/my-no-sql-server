use std::sync::Arc;

use rust_extensions::StopWatch;

use crate::{
    app::{logs::SystemProcess, AppContext},
    db::DbTable,
    persist_operations::data_to_persist::PersistResult,
};

pub enum PersistType {
    Dedicated(Arc<DbTable>),
    Common,
}

pub async fn persist(app: &Arc<AppContext>, persist_type: &PersistType) {
    let is_shutting_down = app.states.is_shutting_down();

    loop {
        let tables = match persist_type {
            PersistType::Dedicated(db_table) => vec![db_table.clone()],
            PersistType::Common => app.db.get_tables_with_common_persist_thread().await,
        };

        let mut has_something_to_persist = false;

        for db_table in tables {
            if let Some(persist_result) = db_table.get_what_to_persist(is_shutting_down).await {
                has_something_to_persist = true;
                let mut sw = StopWatch::new();
                sw.start();
                let result = tokio::spawn(persist_to_blob(
                    app.clone(),
                    db_table.clone(),
                    persist_result,
                ))
                .await;

                sw.pause();

                db_table
                    .update_last_persist_time(result.is_ok(), sw.duration())
                    .await;

                if let Err(err) = result {
                    app.logs.add_fatal_error(
                        Some(db_table.name.to_string()),
                        SystemProcess::PersistOperation,
                        "PersistTimer".to_string(),
                        format!("Can not persist messages {:?}", err),
                    )
                }
            }
        }

        if !has_something_to_persist {
            break;
        }
    }
}

async fn persist_to_blob(
    app: Arc<AppContext>,
    db_table: Arc<DbTable>,
    persist_result: PersistResult,
) {
    match persist_result {
        PersistResult::PersistAttrs => {
            let attrs = db_table.attributes.get_snapshot();
            crate::persist_operations::sync::save_table_attributes(
                app.as_ref(),
                db_table.name.as_str(),
                &attrs,
            )
            .await;
        }
        PersistResult::PersistTable => {
            crate::persist_operations::sync::save_table(app.as_ref(), db_table.as_ref()).await;
        }
        PersistResult::PersistPartition(partition_key) => {
            crate::persist_operations::sync::save_partition(
                app.as_ref(),
                db_table.as_ref(),
                partition_key.as_str(),
            )
            .await;
        }
    }
}
