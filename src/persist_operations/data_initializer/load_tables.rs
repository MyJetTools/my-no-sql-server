use std::{sync::Arc, time::Duration};

use rust_extensions::StopWatch;

use crate::app::AppContext;

pub async fn load_tables(app: Arc<AppContext>) {
    let tables = app.persist_io.get_list_of_tables().await;

    app.init_state.init(tables, app.logs.as_ref()).await;

    tokio::spawn(super::table_list_of_files_loader(app.clone()));

    let mut sw = StopWatch::new();
    sw.start();

    let mut threads = Vec::new();
    for _ in 0..app.settings.init_tabes_threads_amount {
        threads.push(tokio::spawn(load_tables_spawned(app.clone())));
    }

    for thread in threads {
        thread.await.unwrap();
    }

    app.states.set_initialized();

    sw.pause();

    app.logs.add_info(
        None,
        crate::app::logs::SystemProcess::Init,
        "init_tables".to_string(),
        format!("All tables initialized in {:?}", sw.duration()),
    );
}

async fn load_tables_spawned(app: Arc<AppContext>) {
    loop {
        match app.init_state.get_next_table_to_load().await {
            super::load_tasks::ProcessTableToLoad::Process(table_to_load) => {
                let mut sw = StopWatch::new();
                sw.start();
                super::load_table(&app, &table_to_load).await;
                app.init_state
                    .loaded_completed(table_to_load.table_name.as_str())
                    .await;

                sw.pause();
                app.logs.add_info(
                    Some(table_to_load.table_name.to_string()),
                    crate::app::logs::SystemProcess::Init,
                    "init_tables".to_string(),
                    format!(
                        "Table {} is initialized in {:?}",
                        table_to_load.table_name,
                        sw.duration()
                    ),
                );
            }
            super::load_tasks::ProcessTableToLoad::NotReadyYet => {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            super::load_tasks::ProcessTableToLoad::TheEnd => {
                return;
            }
        }
    }
}
