use std::sync::Arc;

use my_http_server::{
    middlewares::controllers::{actions::GetAction, documentation::HttpActionDescription},
    HttpContext, HttpFailResult, HttpOkResult,
};

use crate::app::AppContext;

pub struct MetricsAction {
    app: Arc<AppContext>,
}

impl MetricsAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl GetAction for MetricsAction {
    fn get_route(&self) -> &str {
        "/metrics"
    }
    fn get_description(&self) -> Option<HttpActionDescription> {
        None
    }

    async fn handle_request(&self, _ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let result = self.app.metrics.build();

        HttpOkResult::Content {
            content_type: None,
            content: result.into_bytes(),
        }
        .into()
    }
}
