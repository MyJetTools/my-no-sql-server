use std::sync::Arc;

use rust_extensions::{StopWatch, StringBuilder};

use crate::{
    app::{
        logs::{LogItem, SystemProcess},
        AppContext,
    },
    http::{http_fail::HttpFailResult, http_ok::HttpOkResult},
};

pub async fn get(app: &AppContext) -> Result<HttpOkResult, HttpFailResult> {
    let mut sw = StopWatch::new();
    sw.start();
    let logs = app.logs.get().await;

    return compile_result("logs", logs, sw);
}

pub async fn get_by_table(app: &AppContext, path: &str) -> Result<HttpOkResult, HttpFailResult> {
    let table_name = get_table_name(&path);

    if table_name.is_none() {
        return render_select_table(app).await;
    }

    let table_name = table_name.unwrap();

    let mut sw = StopWatch::new();
    sw.start();
    let logs_result = app.logs.get_by_table_name(table_name).await;

    match logs_result {
        Some(logs) => compile_result("logs by table", logs, sw),
        None => {
            sw.pause();

            let content = format!(
                "Result compiled in: {:?}. No log recods for the table '{}'",
                sw.duration(),
                table_name
            );

            Ok(HttpOkResult::Content {
                content_type: Some(crate::http::web_content_type::WebContentType::Text),
                content: content.into_bytes(),
            })
        }
    }
}

pub async fn get_by_process(app: &AppContext, path: &str) -> Result<HttpOkResult, HttpFailResult> {
    let process_name = get_process_name(&path);

    if process_name.is_none() {
        return render_select_process().await;
    }

    let process_name = process_name.unwrap();

    let process = SystemProcess::parse(process_name);

    if process.is_none() {
        return Ok(HttpOkResult::Content {
            content_type: Some(crate::http::web_content_type::WebContentType::Text),
            content: format!("Invalid process name: {}", process_name).into(),
        });
    }

    let process = process.unwrap();

    let mut sw = StopWatch::new();
    sw.start();
    let logs_result = app.logs.get_by_process(process).await;

    match logs_result {
        Some(logs) => compile_result("logs by process", logs, sw),
        None => {
            sw.pause();

            Ok(HttpOkResult::Content {
                content_type: Some(crate::http::web_content_type::WebContentType::Text),
                content: format!(
                    "Result compiled in: {:?}. No log recods for the process '{}'",
                    sw.duration(),
                    process_name
                )
                .into_bytes(),
            })
        }
    }
}

fn get_table_name(path: &str) -> Option<&str> {
    let segments = path.split('/');

    let mut value = "";
    let mut amount: usize = 0;
    for segment in segments {
        value = segment;
        amount += 1;
    }

    if amount == 4 {
        return Some(value);
    }

    None
}

fn get_process_name(path: &str) -> Option<&str> {
    let segments = path.split('/');

    let mut value = "";
    let mut amount: usize = 0;
    for segment in segments {
        value = segment;
        amount += 1;
    }

    if amount == 4 {
        return Some(value);
    }

    None
}
fn compile_result(
    title: &str,
    logs: Vec<Arc<LogItem>>,
    mut sw: StopWatch,
) -> Result<HttpOkResult, HttpFailResult> {
    let mut sb = StringBuilder::new();

    sb.append_line(
        "<a class='btn btn-outline-secondary btn-sm' href='/logs'>Show All Log records</a>",
    );

    sb.append_line(
        "<a class='btn btn-outline-secondary btn-sm' href='/logs/table'>Show Log records by table</a>",
    );

    sb.append_line(
        "<a class='btn btn-outline-secondary btn-sm' href='/logs/process'>Show Log records by process</a>",
    );

    sb.append_line("<hr/>");

    for log_item in &logs {
        let line = format!(
            "<b style='background:{color}; color:white;'>{level:?}:</b> {dt}</br>",
            color = get_log_level_color(&log_item.as_ref()),
            dt = log_item.date.to_rfc3339(),
            level = log_item.level
        );
        sb.append_line(&line);

        if let Some(table_name) = &log_item.table {
            let line = format!(
                "<b>Table:</b> <a href='/logs/table/{table_name}'>{table_name}</a></br>",
                table_name = table_name
            );
            sb.append_line(line.as_str());
        }

        let line = format!(
            "<b>Process:</b> <a href='/logs/process/{process:?}'>{process:?}</a></br>",
            process = log_item.process
        );
        sb.append_line(line.as_str());

        let line = format!("<b>Process Name:</b> {}</br>", log_item.process_name);
        sb.append_line(line.as_str());

        let line = format!("<b>Msg:</b> {}</br>", log_item.message.as_str());
        sb.append_line(line.as_str());

        if let Some(err_ctx) = &log_item.err_ctx {
            let line = format!("<b>ErrCTX:</b> {}</br>", err_ctx);
            sb.append_line(line.as_str());
        }

        sb.append_line("<hr/>");
    }

    sw.pause();

    let line = format!("Rendered in {:?}", sw.duration());
    sb.append_line(line.as_str());

    Ok(HttpOkResult::Html {
        title: title.to_string(),
        body: sb.to_string_utf8().unwrap(),
    })
}

fn get_log_level_color(item: &LogItem) -> &str {
    match &item.level {
        crate::app::logs::LogLevel::Info => "green",
        crate::app::logs::LogLevel::Error => "orange",
        crate::app::logs::LogLevel::FatalError => "red",
    }
}

async fn render_select_table(app: &AppContext) -> Result<HttpOkResult, HttpFailResult> {
    let mut sb = StringBuilder::new();

    sb.append_line("<h1>Please, select table to show logs</h1>");

    for table in &app.db.get_table_names().await {
        let line = format!(
            "<a class='btn btn-sm btn-outline-primary' href='/logs/table/{table_name}'>{table_name}</a>",
            table_name = table
        );
        sb.append_line(line.as_str())
    }

    Ok(HttpOkResult::Html {
        title: "Select table to show logs".to_string(),
        body: sb.to_string_utf8().unwrap(),
    })
}

async fn render_select_process() -> Result<HttpOkResult, HttpFailResult> {
    let mut sb = StringBuilder::new();

    sb.append_line("<h1>Please, select process to show logs</h1>");

    for process in &SystemProcess::iterate() {
        let line = format!(
            "<a class='btn btn-sm btn-outline-primary' href='/logs/process/{process:?}'>{process:?}</a>",
            process = process
        );
        sb.append_line(line.as_str())
    }

    Ok(HttpOkResult::Html {
        title: "Select table to show logs".to_string(),
        body: sb.to_string_utf8().unwrap(),
    })
}
