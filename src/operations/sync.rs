use my_no_sql_sdk::tcp_contracts::MyNoSqlTcpContract;

use crate::{app::AppContext, data_readers::DataReaderConnection, db_sync::SyncEvent};

pub fn dispatch(app: &AppContext, sync_event: SyncEvent) {
    app.sync.publisher.send(sync_event);
}

pub async fn sync(app: &AppContext, sync_event: &SyncEvent) {
    if let SyncEvent::TableFirstInit(data) = sync_event {
        data.data_reader.set_first_init();

        match &data.data_reader.connection {
            DataReaderConnection::Tcp(tcp_info) => {
                let compressed = tcp_info.is_compressed_data();
                let payloads = crate::data_readers::tcp_connection::tcp_payload_to_send::serialize(
                    sync_event, compressed,
                )
                .await;

                if payloads.len() > 0 {
                    tcp_info.send(payloads.as_slice()).await;
                }
            }
            DataReaderConnection::Http(http_info) => {
                http_info.send(&sync_event).await;
            }
        }

        app.metrics
            .update_pending_to_sync(&data.data_reader.connection);
    } else {
        let data_readers = app
            .data_readers
            .get_subscribed_to_table(sync_event.get_table_name())
            .await;

        if data_readers.is_none() {
            return;
        }
        let data_readers = data_readers.unwrap();

        let mut tcp_contracts_non_compressed: Option<Vec<MyNoSqlTcpContract>> = None;
        let mut tcp_contracts_compressed: Option<Vec<MyNoSqlTcpContract>> = None;

        for data_reader in &data_readers {
            if !data_reader.has_first_init() {
                continue;
            }

            match &data_reader.connection {
                DataReaderConnection::Tcp(connection_info) => {
                    if connection_info.is_compressed_data() {
                        if let Some(payloads) = &tcp_contracts_compressed {
                            connection_info.send(payloads).await;
                        } else {
                            let payloads =
                                crate::data_readers::tcp_connection::tcp_payload_to_send::serialize(
                                    sync_event, true,
                                )
                                .await;

                            if payloads.len() > 0 {
                                connection_info.send(payloads.as_slice()).await;
                                tcp_contracts_compressed = Some(payloads);
                            }
                        }
                    } else {
                        if let Some(to_send) = &tcp_contracts_non_compressed {
                            connection_info.send(to_send).await;
                        } else {
                            let payloads =
                                crate::data_readers::tcp_connection::tcp_payload_to_send::serialize(
                                    sync_event, false,
                                )
                                .await;

                            if payloads.len() > 0 {
                                connection_info.send(&payloads).await;
                                tcp_contracts_non_compressed = Some(payloads);
                            }
                        }
                    }
                }
                DataReaderConnection::Http(http_info) => {
                    http_info.send(&sync_event).await;
                }
            }

            app.metrics.update_pending_to_sync(&data_reader.connection);
        }
    }
}
