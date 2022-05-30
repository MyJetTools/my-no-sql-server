use app::{logs::Logs, AppContext, EventsDispatcherProduction};
use background::{
    gc_db_rows::GcDbRows, gc_http_sessions::GcHttpSessionsTimer, gc_multipart::GcMultipart,
    metrics_updater::MetricsUpdater, persist::PersistTimer,
};
use my_logger::MyLogger;
use my_no_sql_tcp_shared::MyNoSqlReaderTcpSerializer;
use my_tcp_sockets::TcpServer;
use operations::PersistType;
use rust_extensions::MyTimer;
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tcp::{TcpServerEvents, TcpServerLogger};

mod app;
mod grpc;

mod db;
mod db_json_entity;
mod db_operations;
mod db_sync;
mod db_transactions;
mod http;
mod json;
mod persist_operations;
mod tcp;

mod background;
mod data_readers;
mod operations;
mod persist_io;
mod settings_reader;
mod utils;

pub mod mynosqlserver_grpc {
    tonic::include_proto!("mynosqlserver");
}

#[tokio::main]
async fn main() {
    let settings = settings_reader::read_settings().await;

    let settings = Arc::new(settings);

    let mut background_tasks = Vec::new();

    let logs = Arc::new(Logs::new());

    let persist_io = settings.get_persist_io(logs.clone());

    let mut sync_events_dispatcher = EventsDispatcherProduction::new();

    let sync_events_reader = sync_events_dispatcher.get_events_reader();

    let app = AppContext::new(
        logs.clone(),
        settings,
        Box::new(sync_events_dispatcher),
        Arc::new(persist_io),
    );

    let tcp_server_logger = TcpServerLogger::new(app.logs.clone());

    let my_logger_for_tcp_server = MyLogger::new(Arc::new(tcp_server_logger));

    let app = Arc::new(app);

    crate::persist_operations::data_initializer::init_tables(app.clone()).await;
    let mut timer_1s = MyTimer::new(Duration::from_secs(1));

    let mut persist_timer = MyTimer::new(Duration::from_secs(1));

    persist_timer.register_timer(
        "Persist",
        Arc::new(PersistTimer::new(app.clone(), PersistType::Common)),
    );
    timer_1s.register_timer("MetricsUpdated", Arc::new(MetricsUpdater::new(app.clone())));

    let mut timer_10s = MyTimer::new(Duration::from_secs(10));
    timer_10s.register_timer(
        "GcHttpSessions",
        Arc::new(GcHttpSessionsTimer::new(app.clone())),
    );

    let mut timer_30s = MyTimer::new(Duration::from_secs(30));
    timer_30s.register_timer("GcDbRows", Arc::new(GcDbRows::new(app.clone())));
    timer_30s.register_timer("GcMultipart", Arc::new(GcMultipart::new(app.clone())));

    timer_1s.start(app.clone(), app.clone());
    timer_10s.start(app.clone(), app.clone());
    timer_30s.start(app.clone(), app.clone());
    persist_timer.start(app.clone(), app.clone());

    background_tasks.push(tokio::task::spawn(crate::background::sync::start(
        app.clone(),
        sync_events_reader,
    )));

    crate::http::start_up::setup_server(&app);

    let tcp_server = TcpServer::new_with_logger(
        "MyNoSqlReader".to_string(),
        SocketAddr::from(([0, 0, 0, 0], 5125)),
        Arc::new(my_logger_for_tcp_server),
    );

    tcp_server
        .start(
            app.clone(),
            Arc::new(MyNoSqlReaderTcpSerializer::new),
            Arc::new(TcpServerEvents::new(app.clone())),
        )
        .await;

    tokio::task::spawn(crate::grpc::server::start(app.clone(), 5124));

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
