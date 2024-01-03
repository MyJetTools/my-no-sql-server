use std::time::Duration;

use my_logger::LogEventCtx;

pub async fn execute(
    table_name: Option<&str>,
    process_name: &str,
    message: String,
    attempt_no: u8,
) {
    if attempt_no >= 5 {
        panic!("{}", message.as_str());
    }

    let ctx = if let Some(table_name) = table_name {
        LogEventCtx::new()
            .add("tableName", table_name)
            .add("attempt", attempt_no.to_string())
    } else {
        LogEventCtx::new().add("attempt", attempt_no.to_string())
    };

    my_logger::LOGGER.write_error(process_name.to_string(), message, ctx);

    tokio::time::sleep(Duration::from_secs(1)).await;
}
