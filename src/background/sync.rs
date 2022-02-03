use std::sync::Arc;

use tokio::sync::mpsc;

use crate::{
    app::AppContext,
    data_readers::{tcp_connection::TcpPayloadToSend, DataReaderConnection},
    db_sync::SyncEvent,
};

pub async fn start(app: Arc<AppContext>, mut rx: mpsc::UnboundedReceiver<SyncEvent>) {
    while !app.states.is_shutting_down() {
        if let Some(sync_event) = rx.recv().await {
            handle_event(app.as_ref(), &sync_event).await; //TODO - Probably run it through span
        }
    }
}

async fn handle_event(app: &AppContext, sync_event: &SyncEvent) {
    if let SyncEvent::TableFirstInit(data) = sync_event {
        match &data.data_reader.connection {
            DataReaderConnection::Tcp(tcp_info) => {
                if let Some(payload_to_send) = TcpPayloadToSend::parse_from(sync_event).await {
                    tcp_info.send(&payload_to_send).await;
                }
            }
        }
        return;
    }

    let connections = app
        .data_readers
        .get_subscribed_to_table(sync_event.get_table_name())
        .await;

    if connections.is_none() {
        return;
    }
    let connections = connections.unwrap();

    let mut tcp_contracts: Option<TcpPayloadToSend> = None;

    for session in &connections {
        match &session.connection {
            DataReaderConnection::Tcp(info) => {
                if let Some(to_send) = &tcp_contracts {
                    info.send(to_send).await;
                } else {
                    if let Some(to_send) = TcpPayloadToSend::parse_from(&sync_event).await {
                        info.send(&to_send).await;
                        tcp_contracts = Some(to_send);
                    }
                }
            }
        }
    }
}
