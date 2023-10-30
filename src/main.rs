use app::AppContext;
use background::{
    gc_db_rows::GcDbRows, gc_http_sessions::GcHttpSessionsTimer, gc_multipart::GcMultipart,
    metrics_updater::MetricsUpdater, persist::PersistTimer, sync::SyncEventLoop,
};

use my_no_sql_sdk::tcp_contracts::MyNoSqlReaderTcpSerializer;
use my_no_sql_server_core::logs::Logs;
use my_tcp_sockets::TcpServer;
use rust_extensions::MyTimer;
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tcp::TcpServerEvents;
mod zip;

mod app;
mod grpc;

mod persist;

mod db_operations;
mod db_sync;
mod db_transactions;
mod http;
mod persist_operations;
mod tcp;

mod background;
mod data_readers;
mod operations;
mod persist_io;
mod settings_reader;

//TODO - Add Amount of Subscribers to table on UI;

pub mod mynosqlserver_grpc {
    tonic::include_proto!("mynosqlserver");
}

#[tokio::main]
async fn main() {
    let settings = settings_reader::read_settings().await;

    let settings = Arc::new(settings);

    let logs = Arc::new(Logs::new());

    let persist_io = settings.get_persist_io(logs.clone());

    let app = AppContext::new(logs.clone(), settings, persist_io);

    let app = Arc::new(app);

    tokio::spawn(crate::persist_operations::data_initializer::load_tables(
        app.clone(),
    ));

    app.sync
        .register_event_loop(Arc::new(SyncEventLoop::new(app.clone())))
        .await;

    let mut timer_1s = MyTimer::new(Duration::from_secs(1));

    let mut persist_timer = MyTimer::new(Duration::from_secs(1));

    persist_timer.register_timer("Persist", Arc::new(PersistTimer::new(app.clone())));
    timer_1s.register_timer("MetricsUpdated", Arc::new(MetricsUpdater::new(app.clone())));

    let mut timer_10s = MyTimer::new(Duration::from_secs(10));
    timer_10s.register_timer(
        "GcHttpSessions",
        Arc::new(GcHttpSessionsTimer::new(app.clone())),
    );

    let mut timer_30s = MyTimer::new(Duration::from_secs(30));
    timer_30s.register_timer("GcDbRows", Arc::new(GcDbRows::new(app.clone())));
    timer_30s.register_timer("GcMultipart", Arc::new(GcMultipart::new(app.clone())));

    timer_1s.start(app.states.clone(), app.clone());
    timer_10s.start(app.states.clone(), app.clone());
    timer_30s.start(app.states.clone(), app.clone());
    persist_timer.start(app.states.clone(), app.clone());

    app.sync.start(app.states.clone(), app.clone()).await;

    crate::http::start_up::setup_server(&app);

    let tcp_server = TcpServer::new(
        "MyNoSqlReader".to_string(),
        SocketAddr::from(([0, 0, 0, 0], 5125)),
    );

    tcp_server
        .start(
            Arc::new(MyNoSqlReaderTcpSerializer::new),
            Arc::new(TcpServerEvents::new(app.clone())),
            app.states.clone(),
            app.clone(),
        )
        .await;

    tokio::task::spawn(crate::grpc::server::start(app.clone(), 5124));

    signal_hook::flag::register(
        signal_hook::consts::SIGTERM,
        app.states.shutting_down.clone(),
    )
    .unwrap();

    shut_down_task(app).await;
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
