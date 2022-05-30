use std::sync::Arc;

use my_http_server::{
    HttpContext, HttpFailResult, HttpOkResult, HttpOutput, HttpServerMiddleware,
    HttpServerRequestFlow,
};
use rust_extensions::StopWatch;

use crate::{app::AppContext, db::DbTable};

pub struct WriteMetricContext {
    db_table: Arc<DbTable>,
    stop_watch: StopWatch,
}

pub struct RequestMetricsWriter {
    app: Arc<AppContext>,
}

impl RequestMetricsWriter {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }

    async fn get_metrics_context(&self, ctx: &mut HttpContext) -> Option<WriteMetricContext> {
        if let Ok(qs) = ctx.request.get_query_string() {
            if let Some(table_name) = qs.get_optional("tableName") {
                if let Ok(table_name) = table_name.as_string() {
                    if let Some(db_table) = self.app.db.get_table(table_name.as_str()).await {
                        let mut result = WriteMetricContext {
                            db_table,
                            stop_watch: StopWatch::new(),
                        };

                        result.stop_watch.start();
                        return result.into();
                    }
                }
            }
        }

        None
    }
}

#[async_trait::async_trait]
impl HttpServerMiddleware for RequestMetricsWriter {
    async fn handle_request(
        &self,
        ctx: &mut HttpContext,
        get_next: &mut HttpServerRequestFlow,
    ) -> Result<HttpOkResult, HttpFailResult> {
        let metrics_context = self.get_metrics_context(ctx).await;
        let result = get_next.next(ctx).await;

        if let Some(mut metrics_context) = metrics_context {
            metrics_context.stop_watch.pause();

            let (result_code, result_size) = match &result {
                Ok(result) => (
                    result.output.get_status_code(),
                    get_content_size(&result.output),
                ),
                Err(err) => (err.status_code, 0),
            };

            metrics_context
                .db_table
                .request_metrics
                .add_metric(
                    format!("[{}]{}", ctx.request.get_method(), ctx.request.get_path(),),
                    metrics_context.stop_watch.duration(),
                    result_code,
                    result_size,
                )
                .await;
        }

        result
    }
}

fn get_content_size(src: &HttpOutput) -> usize {
    match src {
        HttpOutput::Empty => 0,
        HttpOutput::Content {
            headers: _,
            content_type: _,
            content,
        } => content.len(),
        HttpOutput::Redirect { url: _ } => 0,
    }
}
