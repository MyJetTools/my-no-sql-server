use crate::app::logs::Logs;
use std::time::Duration;

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

    logs.add_error(
        table_name,
        crate::app::logs::SystemProcess::Init,
        process_name.to_string(),
        message,
        Some(format!("Attempt: {}", attempt_no)),
    );

    tokio::time::sleep(Duration::from_secs(1)).await;
}
