use std::{collections::HashMap, time::Duration};

use my_no_sql_server_core::logs::*;

pub async fn execute(
    logs: &Logs,
    table_name: Option<String>,
    process_name: &str,
    message: String,
    attempt_no: u8,
) {
    if attempt_no >= 5 {
        panic!("{}", message.as_str());
    }

    let mut ctx = HashMap::new();

    ctx.insert("attempt".to_string(), attempt_no.to_string());

    logs.add_error(
        table_name,
        SystemProcess::Init,
        process_name.to_string(),
        message,
        Some(ctx),
    );

    tokio::time::sleep(Duration::from_secs(1)).await;
}
