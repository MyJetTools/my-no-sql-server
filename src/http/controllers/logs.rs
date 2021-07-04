use crate::{
    app::{logs::LogItem, AppServices},
    db::{FailOperationResult, OperationResult},
    utils::{StopWatch, StringBuilder},
};

pub async fn get(app: &AppServices) -> Result<OperationResult, FailOperationResult> {
    let mut sw = StopWatch::new();
    sw.start();
    let logs = app.logs.get().await;

    return compile_result(logs, sw);
}

pub async fn get_by_table(
    app: &AppServices,
    path: &str,
) -> Result<OperationResult, FailOperationResult> {
    let table_name = get_table_name(&path);

    let mut sw = StopWatch::new();
    sw.start();
    let logs_result = app.logs.get_by_table_name(table_name).await;

    match logs_result {
        Some(logs) => compile_result(logs, sw),
        None => {
            sw.pause();

            Ok(OperationResult::Text {
                text: format!(
                    "Result compiled in: {:?}. No log recods for the table {}",
                    sw.duration(),
                    table_name
                ),
            })
        }
    }
}

fn get_table_name(path: &str) -> &str {
    let segments = path.split('/');
    return segments.last().unwrap();
}

fn compile_result(
    logs: Vec<LogItem>,
    mut sw: StopWatch,
) -> Result<OperationResult, FailOperationResult> {
    let mut sb = StringBuilder::new();

    for log_item in logs {
        let line = format!("{} {:?}", log_item.date.to_iso_string(), log_item.level);
        sb.append_line(&line);

        let line = format!("Process: {}", log_item.process);
        sb.append_line(line.as_str());

        sb.append_line(log_item.message.as_str());

        if let Some(err_ctx) = log_item.err_ctx {
            let line = format!("ErrCTX: {}", err_ctx);
            sb.append_line(line.as_str());
        }

        sb.append_line("-----------------------------");
    }

    sw.pause();

    let line = format!("Rendered in {:?}", sw.duration());
    sb.append_line(line.as_str());

    Ok(OperationResult::Text {
        text: sb.to_string_utf8().unwrap(),
    })
}
