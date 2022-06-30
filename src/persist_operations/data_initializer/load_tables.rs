use std::sync::Arc;

use rust_extensions::StopWatch;

use crate::app::AppContext;

pub async fn load_tables(app: Arc<AppContext>) {
    let tables = app.persist_io.get_list_of_tables().await;

    app.init_state
        .init_table_names(tables, app.logs.as_ref())
        .await;

    tokio::spawn(super::table_list_of_files_loader(app.clone()));

    let mut sw = StopWatch::new();
    sw.start();

    let mut threads = Vec::new();
    for _ in 0..app.settings.init_threads_amount {
        threads.push(tokio::spawn(super::load_table_files::spawn(app.clone())));
    }

    for thread in threads {
        thread.await.unwrap();
    }

    while let Some((db_table_data, attrs)) = app.init_state.get_table_data_result().await {
        crate::db_operations::write::table::init(app.as_ref(), db_table_data, attrs).await;
    }

    app.states.set_initialized();

    sw.pause();

    app.logs.add_info(
        None,
        crate::app::logs::SystemProcess::Init,
        "init_tables".to_string(),
        format!("All tables initialized in {:?}", sw.duration()),
        None,
    );
}
