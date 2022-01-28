use std::sync::Arc;

use my_http_server::{
    middlewares::controllers::{actions::GetAction, documentation::HttpActionDescription},
    HttpContext, HttpFailResult, HttpOkResult, WebContentType,
};
use rust_extensions::StopWatch;

use crate::app::{logs::SystemProcess, AppContext};

pub struct LogsByProcessAction {
    app: Arc<AppContext>,
}

impl LogsByProcessAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl GetAction for LogsByProcessAction {
    fn get_route(&self) -> &str {
        "/Logs/ByProcess/{process_name}"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        None
    }

    async fn handle_request(&self, ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let process_name = get_process_name(ctx.get_path());

        if process_name.is_none() {
            return render_select_process().await.into();
        }

        let process_name = process_name.unwrap();

        let process = SystemProcess::parse(process_name);

        if process.is_none() {
            return Ok(HttpOkResult::Content {
                content_type: Some(WebContentType::Text),
                content: format!("Invalid process name: {}", process_name).into(),
            });
        }

        let process = process.unwrap();

        let mut sw = StopWatch::new();
        sw.start();
        let logs_result = self.app.logs.get_by_process(process).await;

        match logs_result {
            Some(logs) => super::logs::compile_result("logs by process", logs, sw).into(),
            None => {
                sw.pause();

                Ok(HttpOkResult::Content {
                    content_type: Some(WebContentType::Text),
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

async fn render_select_process() -> HttpOkResult {
    let mut sb = String::new();

    sb.push_str("<h1>Please, select process to show logs</h1>");

    for process in &SystemProcess::iterate() {
        let line = format!(
            "<a class='btn btn-sm btn-outline-primary' href='/logs/process/{process:?}'>{process:?}</a>",
            process = process
        );
        sb.push_str(line.as_str())
    }

    super::super::as_html::build("Select table to show logs", sb.as_str()).into()
}
