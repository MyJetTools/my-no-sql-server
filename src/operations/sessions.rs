use std::sync::Arc;

use my_no_sql_tcp_shared::TcpContract;

use crate::{
    app::AppContext,
    tcp::{ReaderSession, SendPackageError},
};

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

pub async fn subscribe(app: &AppContext, session: &ReaderSession, table_name: &str) -> bool {
    let table = app.db.get_table(table_name).await;

    if table.is_none() {
        println!(
            "{} is subscribing to the table {} which does not exist",
            session.get_name().await,
            table_name
        );

        return false;
    }

    let table = table.unwrap();

    session.subscribe(table_name.to_string()).await;

    let tcp_package = TcpContract::InitTable {
        table_name: table_name.to_string(),
        data: table.as_json().await,
    };
    send_package_and_forget(session, &tcp_package).await;

    return true;
}
