use crate::{app::AppContext, data_readers::DataReaderConnection, db_sync::SyncEvent};

pub fn dispatch(app: &AppContext, sync_event: SyncEvent) {
    app.sync.send(sync_event);
}

pub async fn sync(app: &AppContext, sync_event: &SyncEvent) {
    if let SyncEvent::TableFirstInit(data) = sync_event {
        data.data_reader.set_first_init();

        match &data.data_reader.connection {
            DataReaderConnection::Tcp(tcp_info) => {
                let compressed = tcp_info.is_compressed_data();
                if let Some(payload_to_send) =
                    crate::data_readers::tcp_connection::tcp_payload_to_send::serialize(
                        sync_event, compressed,
                    )
                    .await
                {
                    tcp_info.send(&payload_to_send).await;
                }
            }
            DataReaderConnection::Http(http_info) => {
                http_info.send(&sync_event).await;
            }
        }

        app.metrics
            .update_pending_to_sync(&data.data_reader.connection)
            .await;
    } else {
        let data_readers = app
            .data_readers
            .get_subscribed_to_table(sync_event.get_table_name())
            .await;

        if data_readers.is_none() {
            return;
        }
        let data_readers = data_readers.unwrap();

        let mut tcp_contracts_non_compresed: Option<Vec<u8>> = None;
        let mut tcp_contracts_compressed: Option<Vec<u8>> = None;

        for data_reader in &data_readers {
            if !data_reader.has_first_init() {
                continue;
            }

            match &data_reader.connection {
                DataReaderConnection::Tcp(info) => {
                    if info.is_compressed_data() {
                        if let Some(to_send) = &tcp_contracts_compressed {
                            info.send(to_send).await;
                        } else {
                            if let Some(to_send) =
                                crate::data_readers::tcp_connection::tcp_payload_to_send::serialize(
                                    sync_event, true,
                                )
                                .await
                            {
                                info.send(&to_send).await;
                                tcp_contracts_compressed = Some(to_send);
                            }
                        }
                    } else {
                        if let Some(to_send) = &tcp_contracts_non_compresed {
                            info.send(to_send).await;
                        } else {
                            if let Some(to_send) =
                                crate::data_readers::tcp_connection::tcp_payload_to_send::serialize(
                                    sync_event, false,
                                )
                                .await
                            {
                                info.send(&to_send).await;
                                tcp_contracts_non_compresed = Some(to_send);
                            }
                        }
                    }
                }
                DataReaderConnection::Http(http_info) => {
                    http_info.send(&sync_event).await;
                }
            }

            app.metrics
                .update_pending_to_sync(&data_reader.connection)
                .await;
        }
    }
}
