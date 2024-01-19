use std::sync::Arc;

use my_logger::LogEventCtx;
use my_no_sql_server_core::DbTableWrapper;
use rust_extensions::{date_time::DateTimeAsMicroseconds, StopWatch};

use crate::{app::AppContext, persist::partition_persist_marker::PersistResult};

pub async fn persist(app: &Arc<AppContext>) {
    let is_shutting_down = app.states.is_shutting_down();

    loop {
        let tables = app.db.get_tables().await;

        let mut has_something_to_persist = false;

        for db_table in tables {
            if let Some(persist_result) = app
                .persist_markers
                .get_job_to_persist(
                    db_table.name.as_str(),
                    DateTimeAsMicroseconds::now(),
                    is_shutting_down,
                )
                .await
            {
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

                if result.is_ok() {
                    app.persist_markers
                        .set_persisted(db_table.name.as_str(), sw.duration())
                        .await;
                }

                if let Err(err) = result {
                    my_logger::LOGGER.write_fatal_error(
                        "PersistTimer".to_string(),
                        format!("Can not persist messages {:?}", err),
                        LogEventCtx::new().add("table_name", db_table.name.as_str()),
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
    db_table: Arc<DbTableWrapper>,
    persist_result: PersistResult,
) {
    match persist_result {
        PersistResult::PersistAttrs => {
            let attrs = db_table.get_attributes().await;
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
            crate::persist_operations::sync::save_partition(app.as_ref(), &db_table, partition_key)
                .await;
        }
    }
}
