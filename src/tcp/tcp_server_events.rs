use std::{collections::HashMap, sync::Arc};

use my_no_sql_server_core::logs::*;
use my_no_sql_tcp_shared::{MyNoSqlReaderTcpSerializer, MyNoSqlTcpContract};
use my_tcp_sockets::{tcp_connection::SocketConnection, ConnectionEvent, SocketEventCallback};

use crate::app::AppContext;

pub type MyNoSqlTcpConnection = SocketConnection<MyNoSqlTcpContract, MyNoSqlReaderTcpSerializer>;

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
                connection.send(MyNoSqlTcpContract::Pong).await;
            }
            MyNoSqlTcpContract::Greeting { name } => {
                if let Some(data_reader) = self.app.data_readers.get_tcp(connection.as_ref()).await
                {
                    self.app.logs.add_info(
                        None,
                        SystemProcess::TcpSocket,
                        format!("Connection name update to {}", name),
                        format!("ID: {}", connection.id),
                        None,
                    );
                    data_reader.set_name_as_reader(name.to_string()).await;
                }
            }

            MyNoSqlTcpContract::GreetingFromNode {
                node_location,
                node_version,
                compress,
            } => {
                if let Some(data_reader) = self.app.data_readers.get_tcp(connection.as_ref()).await
                {
                    println!(
                        "Connected node: {}:{}. Compress:{}",
                        node_location, node_version, compress
                    );

                    self.app.logs.add_info(
                        None,
                        SystemProcess::TcpSocket,
                        format!(
                            "Connection to node with location {} and version {}",
                            node_location, node_version
                        ),
                        format!("ID: {}", connection.id),
                        None,
                    );
                    data_reader
                        .set_name_as_node(node_location, node_version, compress)
                        .await;
                }
            }

            MyNoSqlTcpContract::Subscribe { table_name } => {
                if let Some(data_reader) = self.app.data_readers.get_tcp(connection.as_ref()).await
                {
                    let result = crate::operations::data_readers::subscribe(
                        self.app.as_ref(),
                        data_reader,
                        &table_name,
                    )
                    .await;

                    if let Err(err) = result {
                        let session = self.app.data_readers.get_tcp(connection.as_ref()).await;

                        let session_name = if let Some(session) = session {
                            session.get_name().await
                        } else {
                            None
                        };

                        let message =
                            format!("Subscribe to table {} error. Err: {:?}", table_name, err);

                        let mut ctx = HashMap::new();

                        ctx.insert("sessionId".to_string(), connection.id.to_string());
                        if let Some(session_name) = session_name {
                            ctx.insert("sessionName".to_string(), session_name);
                        }

                        self.app.logs.add_error(
                            Some(table_name.to_string()),
                            SystemProcess::TcpSocket,
                            "Subscribe to table".to_string(),
                            message.to_string(),
                            Some(ctx),
                        );

                        connection.send(MyNoSqlTcpContract::Error { message }).await;
                    }
                }
            }

            MyNoSqlTcpContract::SubscribeAsNode(table_name) => {
                if let Some(data_reader) = self.app.data_readers.get_tcp(connection.as_ref()).await
                {
                    let table = self.app.db.get_table(table_name.as_str()).await;

                    if table.is_none() {
                        connection
                            .send(MyNoSqlTcpContract::TableNotFound(table_name))
                            .await;

                        return;
                    }

                    let result = crate::operations::data_readers::subscribe(
                        self.app.as_ref(),
                        data_reader.clone(),
                        &table_name,
                    )
                    .await;

                    if let Err(err) = result {
                        let session = self.app.data_readers.get_tcp(connection.as_ref()).await;

                        let session_name = if let Some(session) = session {
                            session.get_name().await
                        } else {
                            None
                        };

                        let message =
                            format!("Subscribe to table {} error. Err: {:?}", table_name, err);

                        let mut ctx = HashMap::new();

                        ctx.insert("sessionId".to_string(), connection.id.to_string());
                        if let Some(session_name) = session_name {
                            ctx.insert("sessionName".to_string(), session_name);
                        }

                        self.app.logs.add_error(
                            Some(table_name.to_string()),
                            SystemProcess::TcpSocket,
                            "Subscribe to table".to_string(),
                            message.to_string(),
                            Some(ctx),
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
                    .send(MyNoSqlTcpContract::Confirmation { confirmation_id })
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
                        row_keys.iter(),
                    )
                    .await;
                }

                connection
                    .send(MyNoSqlTcpContract::Confirmation { confirmation_id })
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
                    .send(MyNoSqlTcpContract::Confirmation { confirmation_id })
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
                        row_keys.iter(),
                        expiration_time,
                    )
                }

                connection
                    .send(MyNoSqlTcpContract::Confirmation { confirmation_id })
                    .await;
            }

            _ => {}
        }
    }
}

#[async_trait::async_trait]
impl SocketEventCallback<MyNoSqlTcpContract, MyNoSqlReaderTcpSerializer> for TcpServerEvents {
    async fn handle(
        &self,
        connection_event: ConnectionEvent<MyNoSqlTcpContract, MyNoSqlReaderTcpSerializer>,
    ) {
        match connection_event {
            ConnectionEvent::Connected(connection) => {
                self.app.logs.add_info(
                    None,
                    SystemProcess::TcpSocket,
                    "New tcp connection".to_string(),
                    format!("ID: {}", connection.id),
                    None,
                );

                self.app.data_readers.add_tcp(connection).await;
                self.app.metrics.mark_new_tcp_connection();
            }
            ConnectionEvent::Disconnected(connection) => {
                self.app.logs.add_info(
                    None,
                    SystemProcess::TcpSocket,
                    "Disconnect".to_string(),
                    format!("ID: {}", connection.id),
                    None,
                );
                if let Some(data_reader) =
                    self.app.data_readers.remove_tcp(connection.as_ref()).await
                {
                    self.app
                        .metrics
                        .remove_pending_to_sync(&data_reader.connection)
                        .await;
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
