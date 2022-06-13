use std::{sync::Arc, time::Duration};

use crate::{
    app::AppContext,
    persist_operations::data_initializer::load_tasks::{PartitionToLoad, TableToLoad},
};

use super::LoadedTable;

pub async fn load_table_files(
    app: Arc<AppContext>,
    table_to_load: &Arc<TableToLoad>,
) -> Arc<LoadedTable> {
    let loaded_table = Arc::new(LoadedTable::new());

    let mut to_load = Vec::with_capacity(app.settings.init_partitions_threads_amount);

    for _ in 0..app.settings.init_partitions_threads_amount {
        to_load.push(tokio::spawn(load_table_files_loop(
            app.clone(),
            table_to_load.clone(),
            loaded_table.clone(),
        )));
    }

    for handle in to_load {
        handle.await.unwrap();
    }

    loaded_table
}

async fn load_table_files_loop(
    app: Arc<AppContext>,
    table_to_load: Arc<TableToLoad>,
    loaded_table: Arc<LoadedTable>,
) {
    loop {
        match table_to_load.get_next().await {
            Some(next_task) => match next_task {
                PartitionToLoad::Load(file_name) => {
                    super::load_table_file(
                        table_to_load.table_name.as_str(),
                        file_name,
                        &loaded_table,
                        &app,
                    )
                    .await;
                }
                PartitionToLoad::EndOfReading => {
                    break;
                }
            },
            None => {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
    }
}
