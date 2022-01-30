use std::sync::Arc;

use my_no_sql_tcp_shared::{MyNoSqlReaderTcpSerializer, TcpContract};
use my_tcp_sockets::{tcp_connection::SocketConnection, ConnectionEvent, SocketEventCallback};

use crate::{app::AppContext, operations::OperationError};

pub type MyNoSqlTcpConnection = SocketConnection<TcpContract, MyNoSqlReaderTcpSerializer>;

pub struct TcpServerEvents {
    app: Arc<AppContext>,
}

impl TcpServerEvents {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }

    pub async fn handle_incoming_packet(
        &self,
        tcp_contract: TcpContract,
        connection: Arc<MyNoSqlTcpConnection>,
    ) {
        match tcp_contract {
            TcpContract::Ping => {
                connection.send(TcpContract::Pong).await;
            }
            TcpContract::Greeting { name } => {
                if let Some(data_reader) = self.app.data_readers.get_tcp(connection.as_ref()).await
                {
                    data_reader.set_name(name).await;
                }
            }

            TcpContract::Subscribe { table_name } => {
                if let Some(data_reader) = self.app.data_readers.get_tcp(connection.as_ref()).await
                {
                    let result = crate::operations::data_readers::subscribe(
                        self.app.as_ref(),
                        data_reader,
                        &table_name,
                    )
                    .await;

                    if let Err(err) = result {
                        match err {
                            OperationError::TableNotFound => {
                                panic!("Table {} is not found to subscribe", table_name);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

#[async_trait::async_trait]
impl SocketEventCallback<TcpContract, MyNoSqlReaderTcpSerializer> for TcpServerEvents {
    async fn handle(
        &self,
        connection_event: ConnectionEvent<TcpContract, MyNoSqlReaderTcpSerializer>,
    ) {
        match connection_event {
            ConnectionEvent::Connected(connection) => {
                println!("New tcp connection: {}", connection.id);

                self.app.data_readers.add_tcp(connection).await;
            }
            ConnectionEvent::Disconnected(connection) => {
                println!("Connection {} is disconnected", connection.id);
                self.app.data_readers.remove_tcp(connection.as_ref()).await;
            }
            ConnectionEvent::Payload {
                connection,
                payload,
            } => self.handle_incoming_packet(payload, connection).await,
        }
    }
}
