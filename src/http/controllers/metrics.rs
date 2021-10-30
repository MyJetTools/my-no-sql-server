use crate::{
    app::AppContext,
    http::{http_fail::HttpFailResult, http_ok::HttpOkResult},
};

pub fn get(app: &AppContext) -> Result<HttpOkResult, HttpFailResult> {
    let result = HttpOkResult::Text {
        text: app.metrics.build(),
    };

    Ok(result)
}
