use std::{sync::Arc, time::Duration};

use crate::{
    app::{logs::SystemProcess, AppContext, SyncEventsReader},
    data_readers::DataReaderConnection,
    db_sync::SyncEvent,
};

pub async fn start(app: Arc<AppContext>, mut sync_events_reader: SyncEventsReader) {
    let time_out_duration = Duration::from_secs(1);

    while !app.states.is_shutting_down() {
        if let Some(sync_event) = sync_events_reader.get_next_event().await {
            let result =
                tokio::time::timeout(time_out_duration, handle_event(app.as_ref(), &sync_event))
                    .await;

            if let Err(_) = result {
                app.logs.add_fatal_error(
                    None,
                    SystemProcess::ReadersSync,
                    "Sync Loop".to_string(),
                    format!(
                        "Handling event for table {} is timeouted. ",
                        sync_event.get_table_name()
                    ),
                );
            }
        } else {
            app.logs.add_fatal_error(
                None,
                SystemProcess::ReadersSync,
                "Sync Loop".to_string(),
                "Somehow we got empty event".to_string(),
            );
            tokio::time::sleep(time_out_duration).await;
        }
    }
}

async fn handle_event(app: &AppContext, sync_event: &SyncEvent) {
    if let SyncEvent::TableFirstInit(data) = sync_event {
        data.data_reader.set_first_init();

        match &data.data_reader.connection {
            DataReaderConnection::Tcp(tcp_info) => {
                if let Some(payload_to_send) =
                    crate::data_readers::tcp_connection::tcp_payload_to_send::serialize(sync_event)
                        .await
                {
                    tcp_info.send(&payload_to_send).await;
                }
            }
            DataReaderConnection::Http(http_info) => {
                http_info.send(&sync_event).await;
            }
        }
        return;
    }

    let data_readers = app
        .data_readers
        .get_subscribed_to_table(sync_event.get_table_name())
        .await;

    if data_readers.is_none() {
        return;
    }
    let data_readers = data_readers.unwrap();

    let mut tcp_contracts: Option<Vec<u8>> = None;

    for data_reader in &data_readers {
        if !data_reader.has_first_init() {
            continue;
        }

        match &data_reader.connection {
            DataReaderConnection::Tcp(info) => {
                if let Some(to_send) = &tcp_contracts {
                    info.send(to_send).await;
                } else {
                    if let Some(to_send) =
                        crate::data_readers::tcp_connection::tcp_payload_to_send::serialize(
                            sync_event,
                        )
                        .await
                    {
                        info.send(&to_send).await;
                        tcp_contracts = Some(to_send);
                    }
                }
            }
            DataReaderConnection::Http(http_info) => {
                http_info.send(&sync_event).await;
            }
        }
    }
}
