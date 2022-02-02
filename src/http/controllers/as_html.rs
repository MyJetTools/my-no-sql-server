use my_http_server::{HttpOkResult, WebContentType};

pub fn build(title: &str, body: &str) -> HttpOkResult {
    HttpOkResult::Content {
        content_type: Some(WebContentType::Html),
        content: format!(
            r###"<html><head><title>{ver} MyNoSQLServer {title}</title>
            <link href="/css/bootstrap.css" rel="stylesheet" type="text/css" />
            </head><body>{body}</body></html>"###,
            ver = crate::app::APP_VERSION,
        )
        .into_bytes(),
    }
}