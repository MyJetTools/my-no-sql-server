use std::sync::Arc;

use my_no_sql_tcp_shared::TcpContract;

use crate::tcp::{ReaderSession, SendPackageError};

pub async fn disconnect(session: Arc<ReaderSession>) {
    session.disconnect().await;
}

pub async fn send_package(
    session: &ReaderSession,
    tcp_contract: &TcpContract,
) -> Result<(), SendPackageError> {
    session.send_package(tcp_contract).await
}

pub async fn send_package_and_forget(session: &ReaderSession, tcp_contract: &TcpContract) {
    let result = session.send_package(tcp_contract).await;
    if let Err(_) = result {}
}
