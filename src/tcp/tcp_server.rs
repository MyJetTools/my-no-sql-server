use std::{net::SocketAddr, sync::Arc, time::Duration};

use my_no_sql_tcp_shared::{SocketReader, TcpContract};
use rust_extensions::date_time::DateTimeAsMicroseconds;

use tokio::{
    io::{self, AsyncWriteExt, ReadHalf},
    net::{TcpListener, TcpStream},
};

use crate::{app::AppContext, tcp::ReaderSession};

use super::error::ReadSocketError;

pub type ConnectionId = u64;

pub async fn start(app: Arc<AppContext>, addr: SocketAddr) {
    while !app.states.is_initialized() {
        tokio::time::sleep(Duration::from_secs(3)).await;
    }

    app.logs
        .add_info(
            None,
            crate::app::logs::SystemProcess::TcpSocket,
            "Tcp socket is started".to_string(),
            format!("{:?}", addr),
        )
        .await;

    let listener = TcpListener::bind(addr).await.unwrap();

    let mut socket_id: ConnectionId = 0;

    while !app.states.is_shutting_down() {
        let accepted_socket_result = listener.accept().await;

        if let Err(err) = &accepted_socket_result {
            app.logs
                .add_error(
                    None,
                    crate::app::logs::SystemProcess::TcpSocket,
                    "Accept tcp socket".to_string(),
                    "Error occured".to_string(),
                    Some(format!("{:?}", err)),
                )
                .await;
            continue;
        }

        //Safety: We can use unwrap -since we previously checked Err status.
        let (tcp_stream, addr) = accepted_socket_result.unwrap();

        let (read_socket, mut write_socket) = io::split(tcp_stream);

        if app.states.is_shutting_down() {
            write_socket.shutdown().await.unwrap();
            break;
        }

        socket_id += 1;

        let my_sb_session = Arc::new(ReaderSession::new(
            socket_id,
            format! {"{}", addr},
            write_socket,
            app.logs.clone(),
        ));

        app.data_readers.add(my_sb_session.clone()).await;

        app.logs
            .add_info(
                None,
                crate::app::logs::SystemProcess::TcpSocket,
                "Accepted sockets loop".to_string(),
                format!("Connected socket {}. IP: {}", my_sb_session.id, addr),
            )
            .await;

        tokio::task::spawn(process_socket(read_socket, app.clone(), my_sb_session));
    }

    app.logs
        .add_info(
            None,
            crate::app::logs::SystemProcess::TcpSocket,
            "Tcp socket is stopped".to_string(),
            format!("{:?}", addr),
        )
        .await;
}

async fn process_socket(
    read_socket: ReadHalf<TcpStream>,
    app: Arc<AppContext>,
    reader_session: Arc<ReaderSession>,
) {
    let socket_loop_result =
        tokio::task::spawn(socket_loop(read_socket, reader_session.clone())).await;

    let name = reader_session.get_name().await;

    if let Err(err) = socket_loop_result {
        app.logs
            .add_fatal_error(
                crate::app::logs::SystemProcess::TcpSocket,
                "tcp_socket_process".to_string(),
                format!("Socket {} disconnected error: {:?}", name, err),
            )
            .await;
    } else {
        app.logs
            .add_info(
                None,
                crate::app::logs::SystemProcess::TcpSocket,
                format!("Socket {} Processing", name),
                format!(
                    "Socket with Id:{} and name {} is disconnected",
                    reader_session.id, reader_session.ip
                ),
            )
            .await;
    }
    app.data_readers.remove(&reader_session.id).await;

    crate::operations::sessions::disconnect(reader_session).await;
}

async fn socket_loop(
    read_socket: ReadHalf<TcpStream>,
    session: Arc<ReaderSession>,
) -> Result<(), ReadSocketError> {
    let mut socket_reader = SocketReader::new(read_socket);

    loop {
        socket_reader.start_calculating_read_size();
        let tcp_contract = TcpContract::deserialize(&mut socket_reader).await?;

        let now = DateTimeAsMicroseconds::now();
        session
            .metrics
            .increase_read_size(socket_reader.read_size, now)
            .await;

        super::connection::handle_incoming_payload(tcp_contract, session.clone()).await?;
    }
}
