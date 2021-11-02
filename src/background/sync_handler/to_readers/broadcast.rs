use crate::app::{AppContext, NextEventsToHandle};

pub async fn broadcast(app: &AppContext, next_events: &NextEventsToHandle) {
    let connections = app
        .data_readers
        .get_subscribed_to_table(&next_events.table_name)
        .await;

    if connections.is_none() {
        return;
    }
    let connections = connections.unwrap();

    for event in &next_events.events {
        match super::mappers::into_tcp_contract(event) {
            super::mappers::TcpContractsToSend::None => {}
            super::mappers::TcpContractsToSend::Single(tcp_contract) => {
                for session in &connections {
                    crate::operations::sessions::send_package_and_forget(
                        session.as_ref(),
                        &tcp_contract,
                    )
                    .await;
                }
            }
            super::mappers::TcpContractsToSend::Multiple(contracts) => {
                for tcp_contract in contracts {
                    for session in &connections {
                        crate::operations::sessions::send_package_and_forget(
                            session.as_ref(),
                            &tcp_contract,
                        )
                        .await;
                    }
                }
            }
        }
    }
}
