use std::sync::Arc;

use rust_extensions::StopWatch;

use crate::{
    app::{logs::SystemProcess, AppContext},
    db::DbTableWrapper,
    persist::PersistResult,
};

pub async fn persist(app: &Arc<AppContext>) {
    let is_shutting_down = app.states.is_shutting_down();

    loop {
        let tables = app.db.get_tables().await;

        let mut has_something_to_persist = false;

        for db_table_wrapper in tables {
            let job_to_persist = {
                let mut write_access = db_table_wrapper.data.write().await;

                write_access
                    .persist_markers
                    .data_to_persist
                    .get_what_to_persist()
            };

            if let Some(persist_result) = job_to_persist {
                has_something_to_persist = true;
                let mut sw = StopWatch::new();
                sw.start();
                let result = tokio::spawn(persist_to_blob(
                    app.clone(),
                    db_table_wrapper.clone(),
                    persist_result,
                ))
                .await;

                sw.pause();

                if result.is_ok() {
                    let mut write_access = db_table_wrapper.data.write().await;
                    write_access
                        .persist_markers
                        .add_persist_duration(sw.duration());
                }

                if let Err(err) = result {
                    app.logs.add_fatal_error(
                        Some(db_table_wrapper.name.to_string()),
                        SystemProcess::PersistOperation,
                        "PersistTimer".to_string(),
                        format!("Can not persist messages {:?}", err),
                        None,
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
    db_table_wrapper: Arc<DbTableWrapper>,
    persist_result: PersistResult,
) {
    match persist_result {
        PersistResult::PersistAttrs => {
            let attrs = db_table_wrapper.get_table_attributes().await;
            crate::persist_operations::sync_with_retries::save_table_attributes(
                &app,
                db_table_wrapper.name.as_str(),
                &attrs,
            )
            .await;
        }
        PersistResult::PersistTable(persist_moment) => {
            crate::persist_operations::sync_with_retries::save_table(
                &app,
                db_table_wrapper.as_ref(),
                persist_moment,
            )
            .await;
        }
        PersistResult::PersistPartition {
            partition_key,
            persist_moment,
        } => {
            crate::persist_operations::sync_with_retries::save_partition(
                &app,
                db_table_wrapper.as_ref(),
                partition_key.as_str(),
            )
            .await;
        }
        PersistResult::PersistRows {
            partition_key,
            row_keys,
        } => {
            todo!()
        }
    }
}
