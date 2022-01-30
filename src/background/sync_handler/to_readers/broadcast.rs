use crate::{
    app::{AppContext, NextEventsToHandle},
    data_readers::{tcp_connection::TcpPayloadToSend, DataReaderConnection},
    db_sync::SyncEvent,
};

pub async fn broadcast(app: &AppContext, next_events: &NextEventsToHandle) {
    let connections = app
        .data_readers
        .get_subscribed_to_table(&next_events.table_name)
        .await;

    if connections.is_none() {
        return;
    }
    let connections = connections.unwrap();

    for sync_event in &next_events.events {
        if let SyncEvent::TableFirstInit(data) = sync_event {
            match &data.data_reader.connection {
                DataReaderConnection::Tcp(tcp_info) => {
                    if let Some(payload_to_send) = TcpPayloadToSend::parse_from(sync_event).await {
                        tcp_info.send(&payload_to_send).await;
                    }
                }
            }

            continue;
        }

        let mut tcp_contracts: Option<TcpPayloadToSend> = None;

        for session in &connections {
            match &session.connection {
                DataReaderConnection::Tcp(info) => {
                    if let Some(to_send) = &tcp_contracts {
                        info.send(to_send).await;
                    } else {
                        if let Some(to_send) = TcpPayloadToSend::parse_from(sync_event).await {
                            info.send(&to_send).await;
                            tcp_contracts = Some(to_send);
                        }
                    }
                }
            }
        }
    }
}
