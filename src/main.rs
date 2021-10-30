use app::AppContext;
use std::{net::SocketAddr, sync::Arc, time::Duration};

mod app;

mod db;
mod db_json_entity;
mod db_operations;
mod db_sync;
mod db_transactions;
mod http;
mod json;
mod operations;
mod tcp;

mod background;
mod persistence;
mod settings_reader;
mod utils;

#[tokio::main]
async fn main() {
    let (transactions_sender, transactions_receiver) = tokio::sync::mpsc::unbounded_channel();
    let settings = settings_reader::read_settings().await;

    let app = AppContext::new(&settings, Some(transactions_sender));
    let app = Arc::new(app);

    let connection = settings.get_azure_connection();

    let mut background_tasks = Vec::new();

    if let Some(connection) = connection {
        crate::operations::data_initializer::init_tables(app.as_ref(), &connection).await;

        let handler = tokio::task::spawn(crate::background::blob_persistence::start(
            app.clone(),
            connection,
        ));

        background_tasks.push(handler);
    }

    //background_tasks.push(tokio::task::spawn(tcp_server::start(app.clone(), "*.5125")));

    //   background_tasks.push(tokio::task::spawn(data_readers_broadcast::start(
    //        app.clone(),
    //        data_readers_reciever,
    //    )));

    background_tasks.push(tokio::task::spawn(
        crate::background::metrics_updater::start(app.clone()),
    ));

    background_tasks.push(tokio::task::spawn(
        crate::background::dead_data_readers_gc::start(app.clone()),
    ));

    background_tasks.push(tokio::task::spawn(crate::background::data_gc::start(
        app.clone(),
    )));

    background_tasks.push(tokio::task::spawn(
        crate::background::transactions_handler::start(app.clone(), transactions_receiver),
    ));

    tokio::task::spawn(http::http_server::start(
        app.clone(),
        SocketAddr::from(([0, 0, 0, 0], 5123)),
    ));

    tokio::task::spawn(tcp::tcp_server::start(
        app.clone(),
        SocketAddr::from(([0, 0, 0, 0], 5125)),
    ));

    signal_hook::flag::register(
        signal_hook::consts::SIGTERM,
        app.states.shutting_down.clone(),
    )
    .unwrap();

    shut_down_task(app).await;

    for background_task in background_tasks.drain(..) {
        background_task.await.unwrap();
    }
}

async fn shut_down_task(app: Arc<AppContext>) {
    let duration = Duration::from_secs(1);

    while !app.states.is_shutting_down() {
        tokio::time::sleep(duration).await;
    }

    println!("Shut down detected. Waiting for 1 second to deliver all messages");
    tokio::time::sleep(duration).await;

    crate::operations::shutdown::execute(app.as_ref()).await;
}
