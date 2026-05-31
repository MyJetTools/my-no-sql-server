use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpServerMiddleware};

use crate::app::AppContext;

// Counts every incoming request into the write-traffic statistics:
//   * +1 to write_payloads_per_second (request count)
//   * +Content-Length to write_bytes_per_second (payload size)
// Body length is taken from the `Content-Length` header so the body itself
// is never read here and stays available for the controllers downstream.
pub struct StatisticsMiddleware {
    app: Arc<AppContext>,
}

impl StatisticsMiddleware {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl HttpServerMiddleware for StatisticsMiddleware {
    async fn handle_request(
        &self,
        ctx: &mut HttpContext,
    ) -> Option<Result<HttpOkResult, HttpFailResult>> {
        let body_len = get_content_length(ctx);
        self.app.write_payloads_per_second.increase(1);
        self.app.write_bytes_per_second.increase(body_len);

        // When the writer replays the `session` id issued during the Ping
        // handshake, attribute the request to that writer exactly. Requests
        // without the header are simply not attributed (shown as 0).
        if let Some(session) = get_session(ctx) {
            self.app.writers_traffic.increase(session, body_len);
        }

        None
    }
}

fn get_session(ctx: &HttpContext) -> Option<&str> {
    use my_http_server::HttpRequestHeaders;

    ctx.request
        .get_headers()
        .try_get_case_insensitive_as_str("session")
        .ok()
        .flatten()
        .filter(|value| !value.is_empty())
}

fn get_content_length(ctx: &HttpContext) -> usize {
    use my_http_server::HttpRequestHeaders;

    ctx.request
        .get_headers()
        .try_get_case_insensitive_as_str("content-length")
        .ok()
        .flatten()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(0)
}
