use std::sync::Arc;

use tokio::{
    io::{self, AsyncReadExt, ReadHalf},
    net::{TcpListener, TcpStream},
};

use crate::{app::AppServices, data_readers::data_reader::DataReader, date_time::MyDateTime};

use super::{data_reader_contract::DataReaderContract, socket_read_buffer::SocketReadBuffer};

pub async fn start(app: Arc<AppServices>) {
    let listener = TcpListener::bind("0.0.0.0:5125").await.unwrap();
    let mut id: u64 = 0;

    loop {
        let (tcp_stream, addr) = listener.accept().await.unwrap();

        let (read_socket, write_socket) = io::split(tcp_stream);

        id += 1;

        let data_reader = Arc::new(DataReader::new(id, format! {"{}", addr}, write_socket));

        app.data_readers.add(data_reader.clone()).await;

        app.logs
            .add_info(
                None,
                crate::app::logs::SystemProcess::ServerSocket,
                "Accept sockets loop".to_string(),
                format!("Connected socket: {}", addr),
            )
            .await;

        tokio::task::spawn(process_socket(read_socket, app.clone(), data_reader));
    }
}

async fn process_socket(
    mut read_socket: ReadHalf<TcpStream>,
    app: Arc<AppServices>,
    data_reader: Arc<DataReader>,
) {
    let socket_result = socket_loop(&mut read_socket, app.as_ref(), data_reader.as_ref()).await;

    if let Err(err) = socket_result {
        app.logs
            .add_info(
                None,
                crate::app::logs::SystemProcess::ServerSocket,
                format!("Socket {} Processing", data_reader.to_string().await),
                format!("Disconnected: Reason: {}", err),
            )
            .await;
    }

    app.data_readers.disconnect(data_reader.id).await;
}

async fn socket_loop(
    read_socket: &mut ReadHalf<TcpStream>,
    app: &AppServices,
    data_reader: &DataReader,
) -> Result<(), String> {
    let mut buffer = SocketReadBuffer::new(1024 * 1024 * 5);

    loop {
        let write_slice = buffer.borrow_to_write();

        if write_slice.is_none() {
            return Err(
                "Socket has no left buffer to read incoming data. Disconnecting it".to_string(),
            );
        }

        let read_result = read_socket.read(&mut write_slice.unwrap()).await;

        if let Err(err) = read_result {
            let reason = format!("Error reading from the socket. Err: {:?}", err);
            return Err(reason);
        }

        if let Ok(read_size) = read_result {
            if read_size == 0 {
                return Err("Socket has 0 incoming data. Disconnecting it".to_string());
            }
            buffer.commit_written_size(read_size);
            process_incoming_data(app, data_reader, &mut buffer).await?;
        }
    }
}

async fn process_incoming_data(
    app: &AppServices,
    data_reader: &DataReader,
    socket_buffer_reader: &mut SocketReadBuffer,
) -> Result<(), String> {
    loop {
        let parse_result = DataReaderContract::deserialize(socket_buffer_reader)?;

        match parse_result {
            Some(contract) => {
                socket_buffer_reader.confirm_read_package();
                handle_incoming_package(app, data_reader, contract).await?;
            }
            None => {
                socket_buffer_reader.reset_read_pos();
                return Ok(());
            }
        }
    }
}

async fn handle_incoming_package(
    app: &AppServices,
    data_reader: &DataReader,
    contract: DataReaderContract,
) -> Result<(), String> {
    let now = MyDateTime::utc_now();
    data_reader.last_incoming_package.update(now);

    match contract {
        DataReaderContract::Ping => {
            data_reader
                .send_package(None, DataReaderContract::Pong.serialize().as_slice())
                .await;
        }

        DataReaderContract::Greeting { name } => {
            data_reader.set_socket_name(name).await;
            app.logs
                .add_info(
                    None,
                    crate::app::logs::SystemProcess::ServerSocket,
                    format!("handle_incoming_package for connection {}", data_reader.id),
                    format!(
                        "Changing the name for the connection: {}",
                        data_reader.to_string().await
                    ),
                )
                .await;
        }

        DataReaderContract::Subscribe { table_name } => {
            app.post_command_to_data_readers(
                super::data_readers_broadcast::DataReadersCommand::Subscribe {
                    table_name,
                    connection_id: data_reader.id,
                },
            )
            .await
        }

        _ => {
            return Err(format!("Unsupported packet: {:?}", contract));
        }
    }

    Ok(())
}
