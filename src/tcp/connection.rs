use std::sync::Arc;

use my_no_sql_tcp_shared::TcpContract;

use crate::app::AppContext;

use super::{error::ReadSocketError, ReaderSession};

pub async fn handle_incoming_payload(
    app: &AppContext,
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
            let result =
                crate::operations::sessions::subscribe(app, session.as_ref(), &table_name).await;

            if !result {
                println!("Can not subscribe to the table {}", table_name.as_str());
                return Err(ReadSocketError::Other(format!(
                    "Can not subscribe to the table {}",
                    table_name,
                )));
            }
            Ok(())
        }
        _ => return Ok(()),
    }
}
