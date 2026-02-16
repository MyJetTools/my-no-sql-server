use std::{net::SocketAddr, sync::Arc};

use my_http_server::controllers::swagger::SwaggerMiddleware;
use my_http_server::{HttpConnectionsCounter, MyHttpServer};

use crate::app::AppContext;

pub fn setup_server(app: &Arc<AppContext>) -> HttpConnectionsCounter {
    let http_port = SocketAddr::from(([0, 0, 0, 0], 5123));
    println!("Starting HTTP server at Tcp({:?})", http_port);
    let mut http_server = MyHttpServer::new(http_port);

    let mut unix_socket_http_server = if let Some(mut unix_socket) = app.use_unix_socket.clone() {
        unix_socket.append_segment(crate::consts::WRITER_UNIX_SOCKET_NAME);

        println!("Starting HTTP server at Unix({:?})", unix_socket.as_str());
        let http_server = MyHttpServer::new_as_unix_socket(unix_socket.into_string());
        Some(http_server)
    } else {
        None
    };

    let controllers = Arc::new(crate::http_server::controllers::builder::build(app));

    let swagger_middleware = SwaggerMiddleware::new(
        controllers.clone(),
        "MyNoSqlServer".to_string(),
        crate::app::APP_VERSION.to_string(),
    );
    let swagger_middleware = Arc::new(swagger_middleware);

    let static_files_middleware = Arc::new(
        my_http_server::StaticFilesMiddleware::new()
            .add_index_file("index.html")
            .add_index_path("/data"),
    );

    if let Some(unix_socket_http_server) = unix_socket_http_server.as_mut() {
        unix_socket_http_server.add_middleware(swagger_middleware.clone());
        unix_socket_http_server.add_middleware(controllers.clone());
        unix_socket_http_server.add_middleware(static_files_middleware.clone());
    }

    http_server.add_middleware(swagger_middleware);
    http_server.add_middleware(controllers);
    http_server.add_middleware(static_files_middleware);

    http_server.start(app.states.clone(), my_logger::LOGGER.clone());

    if let Some(unix_socket_http_server) = unix_socket_http_server.as_mut() {
        unix_socket_http_server.start(app.states.clone(), my_logger::LOGGER.clone());
    }

    http_server.get_http_connections_counter()
}
