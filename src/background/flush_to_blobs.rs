use std::{sync::Arc, time::Duration};

use crate::{app::AppContext, persist_operations::PersistResult};

pub async fn start(app: Arc<AppContext>) {
    let one_sec = Duration::from_secs(1);
    while !app.states.is_initialized() {
        tokio::time::sleep(one_sec).await;
    }

    println!("Persistence loop is started");

    loop {
        let result = tokio::spawn(iteration(app.clone())).await;

        if let Err(err) = result {
            println!("flush_to_blobs_err: {:?}", err);
        }

        tokio::time::sleep(one_sec).await;
    }
}

async fn iteration(app: Arc<AppContext>) {
    let is_shutting_down = app.states.is_shutting_down();

    let tables = app.db.get_tables().await;

    for db_table in tables {
        if let Some(persist_result) = db_table.get_what_to_persist(is_shutting_down).await {
            match persist_result {
                PersistResult::PersistAttrs => {
                    crate::operations::persist::io_with_cache::save_table_attributes(
                        app.as_ref(),
                        db_table.as_ref(),
                    )
                    .await;
                }
                PersistResult::PersistTable => {
                    crate::operations::persist::persist_table::execute(
                        app.as_ref(),
                        db_table.as_ref(),
                    )
                    .await;
                }
                PersistResult::PersistPartition(partition_key) => {
                    crate::operations::persist::persist_partition::execute(
                        app.as_ref(),
                        db_table.as_ref(),
                        partition_key.as_str(),
                    )
                    .await;
                }
            }
        }
    }
}
