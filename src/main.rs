use app::AppContext;
use my_http_server::{middlewares::swagger::SwaggerMiddleware, MyHttpServer};
use std::{net::SocketAddr, sync::Arc, time::Duration};

mod app;
mod grpc;

mod blob_operations;
mod db;
mod db_json_entity;
mod db_operations;
mod db_sync;
mod db_transactions;
mod http;
mod json;
mod operations;
mod rows_with_expiration;
mod tcp;

mod background;
mod persistence;
mod settings_reader;
mod utils;

use my_app_insights::AppInsightsTelemetry;

pub mod mynosqlserver_grpc {
    tonic::include_proto!("mynosqlserver");
}

#[tokio::main]
async fn main() {
    let (transactions_sender, transactions_receiver) = tokio::sync::mpsc::unbounded_channel();
    let settings = settings_reader::read_settings().await;

    let mut background_tasks = Vec::new();

    let telemetry_writer = Arc::new(AppInsightsTelemetry::new());

    let connection = if let Some(mut connection) = settings.get_azure_connection() {
        connection.telemetry = Some(telemetry_writer.clone());

        Some(Arc::new(connection))
    } else {
        None
    };

    let app = AppContext::new(&settings, Some(transactions_sender));

    let app = Arc::new(app);

    if let Some(connection) = &connection {
        crate::operations::data_initializer::init_tables(
            app.clone(),
            connection.clone(),
            settings.init_threads_amount,
        )
        .await;

        let handler = tokio::task::spawn(crate::background::flush_to_blobs::start(
            app.clone(),
            connection.clone(),
        ));

        background_tasks.push(handler);
    }

    background_tasks.push(tokio::task::spawn(
        crate::background::metrics_updater::start(app.clone()),
    ));

    background_tasks.push(tokio::task::spawn(
        crate::background::dead_data_readers_gc::start(app.clone()),
    ));

    background_tasks.push(tokio::task::spawn(crate::background::data_gc::start(
        app.clone(),
    )));

    background_tasks.push(tokio::task::spawn(crate::background::sync_handler::start(
        app.clone(),
        transactions_receiver,
    )));

    background_tasks.push(tokio::task::spawn(
        crate::background::db_rows_expirator::start(app.clone()),
    ));

    background_tasks.push(tokio::task::spawn(
        crate::background::gc::gc_partitions::start(app.clone()),
    ));

    let mut http_server = MyHttpServer::new(SocketAddr::from(([0, 0, 0, 0], 5123)));

    let controllers = Arc::new(crate::http::controllers::builder::build(app.clone()));

    let swagger_middleware = SwaggerMiddleware::new(
        controllers.clone(),
        "MyNoSqlServer".to_string(),
        crate::app::APP_VERSION.to_string(),
    );

    http_server.add_middleware(Arc::new(swagger_middleware));
    http_server.add_middleware(controllers);

    tokio::task::spawn(tcp::tcp_server::start(
        app.clone(),
        SocketAddr::from(([0, 0, 0, 0], 5125)),
    ));

    http_server.start(app.clone());

    tokio::task::spawn(crate::grpc::server::start(app.clone(), 5124));

    signal_hook::flag::register(
        signal_hook::consts::SIGTERM,
        app.states.shutting_down.clone(),
    )
    .unwrap();

    telemetry_writer
        .start(app.states.shutting_down.clone())
        .await;

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
