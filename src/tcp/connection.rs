use std::sync::Arc;

use my_no_sql_tcp_shared::TcpContract;

use super::{error::ReadSocketError, ReaderSession};

pub async fn handle_incoming_payload(
    tcp_contract: TcpContract,
    session: Arc<ReaderSession>,
) -> Result<(), ReadSocketError> {
    match tcp_contract {
        TcpContract::Ping => {
            crate::operations::sessions::send_package(session.as_ref(), &TcpContract::Pong).await?;

            Ok(())
        }
        TcpContract::Greeting { name } => {
            session.set_name(name).await;
            Ok(())
        }

        TcpContract::Subscribe { table_name } => {
            session.subscribe(table_name).await;
            Ok(())
        }
        _ => return Ok(()),
    }
}
