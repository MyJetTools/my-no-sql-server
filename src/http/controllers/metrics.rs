use crate::{
    app::AppServices,
    http::{http_fail::HttpFailResult, http_ok::HttpOkResult},
};

pub fn get(app: &AppServices) -> Result<HttpOkResult, HttpFailResult> {
    let result = HttpOkResult::Text {
        text: app.metrics.build(),
    };

    Ok(result)
}
