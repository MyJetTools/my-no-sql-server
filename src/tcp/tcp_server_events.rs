use std::sync::Arc;

use my_logger::LogEventCtx;
use my_no_sql_sdk::tcp_contracts::{MyNoSqlReaderTcpSerializer, MyNoSqlTcpContract};
use my_tcp_sockets::{tcp_connection::TcpSocketConnection, ConnectionEvent, SocketEventCallback};

use crate::{app::AppContext, data_readers::tcp_connection::ReaderName};

pub type MyNoSqlTcpConnection =
    TcpSocketConnection<MyNoSqlTcpContract, MyNoSqlReaderTcpSerializer, ()>;

pub struct TcpServerEvents {
    app: Arc<AppContext>,
}

impl TcpServerEvents {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }

    pub async fn handle_incoming_packet(
        &self,
        tcp_contract: MyNoSqlTcpContract,
        connection: Arc<MyNoSqlTcpConnection>,
    ) {
        match tcp_contract {
            MyNoSqlTcpContract::Ping => {
                connection.send(&MyNoSqlTcpContract::Pong).await;
            }
            MyNoSqlTcpContract::Greeting { name } => {
                my_logger::LOGGER.write_info(
                    "GreetingTcpMessage",
                    "New tcp connection",
                    LogEventCtx::new()
                        .add("Id", connection.id.to_string())
                        .add("Name", name.as_str()),
                );

                self.app
                    .data_readers
                    .add_tcp(connection, ReaderName::AsReader(name), false)
                    .await;
            }

            MyNoSqlTcpContract::GreetingFromNode {
                node_location,
                node_version,
                compress,
            } => {
                let name = ReaderName::AsNode {
                    location: node_location,
                    version: node_version,
                };
                self.app
                    .data_readers
                    .add_tcp(connection, name, compress)
                    .await;
            }

            MyNoSqlTcpContract::Subscribe { table_name } => {
                if let Some(data_reader) = self.app.data_readers.get_tcp(connection.as_ref()).await
                {
                    let result = crate::operations::data_readers::subscribe(
                        &self.app,
                        data_reader,
                        &table_name,
                    )
                    .await;

                    if let Err(err) = result {
                        let session = self.app.data_readers.get_tcp(connection.as_ref()).await;

                        let session_name = if let Some(session) = session {
                            session.get_name().to_string()
                        } else {
                            "".to_string()
                        };

                        let message = format!("Subscribe to table error. Err: {:?}", err);

                        my_logger::LOGGER.write_info(
                            "GreetingTcpMessage",
                            message.as_str(),
                            LogEventCtx::new()
                                .add("sessionId", connection.id.to_string())
                                .add("Name", session_name)
                                .add("TableName", table_name),
                        );

                        connection
                            .send(&MyNoSqlTcpContract::Error { message })
                            .await;
                    }
                }
            }

            MyNoSqlTcpContract::SubscribeAsNode(table_name) => {
                if let Some(data_reader) = self.app.data_readers.get_tcp(connection.as_ref()).await
                {
                    let table = self.app.db.get_table(table_name.as_str()).await;

                    if table.is_none() {
                        connection
                            .send(&MyNoSqlTcpContract::TableNotFound(table_name))
                            .await;

                        return;
                    }

                    let result = crate::operations::data_readers::subscribe(
                        &self.app,
                        data_reader.clone(),
                        &table_name,
                    )
                    .await;

                    if let Err(err) = result {
                        let session = self.app.data_readers.get_tcp(connection.as_ref()).await;

                        let session_name = if let Some(session) = session {
                            session.get_name().to_string()
                        } else {
                            "".to_string()
                        };

                        let message =
                            format!("Subscribe to table {} error. Err: {:?}", table_name, err);

                        my_logger::LOGGER.write_info(
                            "SubscribeToTableAsNode",
                            message,
                            LogEventCtx::new()
                                .add("sessionId", connection.id.to_string())
                                .add("name", session_name)
                                .add("tableName", table_name),
                        );
                    }
                }
            }

            MyNoSqlTcpContract::Unsubscribe(table_name) => {
                if let Some(data_reader) = self.app.data_readers.get_tcp(connection.as_ref()).await
                {
                    data_reader.unsubscribe(table_name.as_str()).await;
                }
            }

            MyNoSqlTcpContract::UpdatePartitionsLastReadTime {
                confirmation_id,
                table_name,
                partitions,
            } => {
                let db_table = self.app.db.get_table(table_name.as_str()).await;

                if let Some(db_table) = db_table {
                    crate::db_operations::update_partitions_last_read_time(
                        &db_table,
                        partitions.iter(),
                    )
                    .await;
                }

                connection
                    .send(&MyNoSqlTcpContract::Confirmation { confirmation_id })
                    .await;
            }

            MyNoSqlTcpContract::UpdateRowsLastReadTime {
                confirmation_id,
                table_name,
                partition_key,
                row_keys,
            } => {
                let db_table = self.app.db.get_table(table_name.as_str()).await;

                if let Some(db_table) = db_table {
                    crate::db_operations::update_row_keys_last_read_access_time(
                        &db_table,
                        &partition_key,
                        row_keys.iter().map(|x| x.as_str()),
                    )
                    .await;
                }

                connection
                    .send(&MyNoSqlTcpContract::Confirmation { confirmation_id })
                    .await;
            }

            MyNoSqlTcpContract::UpdatePartitionsExpirationTime {
                confirmation_id,
                table_name,
                partitions,
            } => {
                let db_table = self.app.db.get_table(table_name.as_str()).await;

                if let Some(db_table) = &db_table {
                    for (partition_key, set_expiration_time) in partitions {
                        crate::db_operations::update_partition_expiration_time(
                            &self.app,
                            db_table,
                            &partition_key,
                            set_expiration_time,
                        )
                    }
                }

                connection
                    .send(&MyNoSqlTcpContract::Confirmation { confirmation_id })
                    .await;
            }
            MyNoSqlTcpContract::UpdateRowsExpirationTime {
                confirmation_id,
                table_name,
                partition_key,
                row_keys,
                expiration_time,
            } => {
                let db_table = self.app.db.get_table(table_name.as_str()).await;

                if let Some(db_table) = &db_table {
                    crate::db_operations::update_rows_expiration_time(
                        &self.app,
                        db_table,
                        &partition_key,
                        row_keys.iter().map(|x| x.as_str()),
                        expiration_time,
                    )
                }

                connection
                    .send(&MyNoSqlTcpContract::Confirmation { confirmation_id })
                    .await;
            }

            _ => {}
        }
    }
}

#[async_trait::async_trait]
impl SocketEventCallback<MyNoSqlTcpContract, MyNoSqlReaderTcpSerializer, ()> for TcpServerEvents {
    async fn handle(
        &self,
        connection_event: ConnectionEvent<MyNoSqlTcpContract, MyNoSqlReaderTcpSerializer, ()>,
    ) {
        match connection_event {
            ConnectionEvent::Connected(_connection) => {
                println!("New connection");
                self.app.metrics.mark_new_tcp_connection();
            }
            ConnectionEvent::Disconnected(connection) => {
                let name = if let Some(data_reader) =
                    self.app.data_readers.get_tcp(connection.as_ref()).await
                {
                    data_reader.get_name().to_string()
                } else {
                    "".to_string()
                };

                my_logger::LOGGER.write_info(
                    "TcpConnection",
                    "Disconnected",
                    LogEventCtx::new()
                        .add("id", connection.id.to_string())
                        .add("Name", name),
                );
                if let Some(data_reader) =
                    self.app.data_readers.remove_tcp(connection.as_ref()).await
                {
                    self.app
                        .metrics
                        .remove_pending_to_sync(&data_reader.connection);
                }
                self.app.metrics.mark_new_tcp_disconnection();
            }
            ConnectionEvent::Payload {
                connection,
                payload,
            } => self.handle_incoming_packet(payload, connection).await,
        }
    }
}
