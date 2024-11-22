use app::AppContext;
use background::{
    gc_db_rows::GcDbRows, gc_http_sessions::GcHttpSessionsTimer, gc_multipart::GcMultipart,
    metrics_updater::MetricsUpdater, persist::PersistTimer, sync::SyncEventLoop, BackupTimer,
};

use my_no_sql_sdk::core::rust_extensions::MyTimer;
use my_no_sql_sdk::tcp_contracts::MyNoSqlTcpSerializerFactory;
use my_tcp_sockets::TcpServer;
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tcp::TcpServerEvents;
mod zip;

mod app;
mod grpc;
mod persist_markers;
mod sqlite_repo;

mod db_operations;
mod db_sync;
mod db_transactions;
mod http;
mod scripts;
mod tcp;

mod background;
mod data_readers;
mod operations;
mod settings_reader;

//TODO - Add Amount of Subscribers to table on UI;

pub mod mynosqlserver_grpc {
    tonic::include_proto!("mynosqlserver");
}

#[global_allocator]
static ALLOC: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

#[tokio::main]
async fn main() {
    let settings = settings_reader::read_settings().await;

    let settings = Arc::new(settings);

    let app = AppContext::new(settings).await;

    let app = Arc::new(app);

    tokio::spawn(crate::operations::init::load_tables(app.clone()));

    let http_connections_counter = crate::http::start_up::setup_server(&app);

    app.sync
        .register_event_loop(Arc::new(SyncEventLoop::new(app.clone())))
        .await;

    let tcp_server = TcpServer::new(
        "MyNoSqlReader".to_string(),
        SocketAddr::from(([0, 0, 0, 0], 5125)),
    );

    let mut timer_1s = MyTimer::new(Duration::from_secs(1));

    let mut persist_timer = MyTimer::new(Duration::from_secs(1));

    persist_timer.register_timer("Persist", Arc::new(PersistTimer::new(app.clone())));
    timer_1s.register_timer(
        "MetricsUpdated",
        Arc::new(MetricsUpdater::new(
            app.clone(),
            http_connections_counter,
            tcp_server.threads_statistics.clone(),
        )),
    );

    let mut timer_10s = MyTimer::new(Duration::from_secs(10));
    timer_10s.register_timer(
        "GcHttpSessions",
        Arc::new(GcHttpSessionsTimer::new(app.clone())),
    );

    let mut timer_30s = MyTimer::new(Duration::from_secs(30));
    timer_30s.register_timer("GcDbRows", Arc::new(GcDbRows::new(app.clone())));
    timer_30s.register_timer("GcMultipart", Arc::new(GcMultipart::new(app.clone())));

    timer_1s.start(app.states.clone(), my_logger::LOGGER.clone());
    timer_10s.start(app.states.clone(), my_logger::LOGGER.clone());
    timer_30s.start(app.states.clone(), my_logger::LOGGER.clone());
    persist_timer.start(app.states.clone(), my_logger::LOGGER.clone());

    let mut backup_timer = MyTimer::new(Duration::from_secs(60));

    backup_timer.register_timer("BackupDb", Arc::new(BackupTimer::new(app.clone())));

    backup_timer.start(app.states.clone(), my_logger::LOGGER.clone());

    app.sync.start(app.states.clone()).await;

    tcp_server
        .start(
            Arc::new(MyNoSqlTcpSerializerFactory),
            Arc::new(TcpServerEvents::new(app.clone())),
            app.states.clone(),
            my_logger::LOGGER.clone(),
        )
        .await;

    tokio::task::spawn(crate::grpc::server::start(app.clone(), 5124));

    app.states.wait_until_shutdown().await;

    crate::operations::shutdown(&app).await;
}
