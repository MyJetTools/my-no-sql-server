use app::AppServices;
use data_readers::{data_readers_broadcast, tcp_server};
use std::sync::Arc;
use tokio::sync::mpsc;

mod app;
mod data_readers;
mod date_time;
mod db;
mod db_operations;
mod db_transactions;
mod http;
mod json;

mod persistence;
mod settings_reader;
mod timers;
mod utils;

#[tokio::main]
async fn main() {
    let settings = settings_reader::read_settings().await;

    let (data_readers_sender, data_readers_reciever) = mpsc::unbounded_channel();
    let app = AppServices::new(settings, data_readers_sender);
    let app = Arc::new(app);

    persistence::tables_initializer::init_tables(app.as_ref()).await;

    let mut background_tasks = Vec::new();

    background_tasks.push(tokio::task::spawn(tcp_server::start(app.clone())));

    background_tasks.push(tokio::task::spawn(data_readers_broadcast::start(
        app.clone(),
        data_readers_reciever,
    )));

    let connection = app.get_azure_connection();

    if let Some(azure_connection) = connection {
        let handler = tokio::task::spawn(crate::timers::blob_operations::blob_persistence::start(
            app.clone(),
            azure_connection.clone(),
        ));

        background_tasks.push(handler);
    }

    background_tasks.push(tokio::task::spawn(crate::timers::metrics_updater::start(
        app.clone(),
    )));

    background_tasks.push(tokio::task::spawn(
        crate::timers::dead_data_readers_gc::start(app.clone()),
    ));

    http::http_server::start(app).await;

    for background_task in background_tasks.drain(..) {
        background_task.await.unwrap();
    }
}
