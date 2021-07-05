use std::collections::HashMap;

use super::{
    http_ctx::HttpContext, http_fail::HttpFailResult, http_ok::HttpOkResult, static_files,
    web_content_type::WebContentType,
};

pub async fn handle_request(path: &str, ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult> {
    if path == "/swagger" {
        return static_files::serve_path("/swagger/index.html").await;
    }

    let result = static_files::serve_path(path).await;
    if path.ends_with("swagger.json") {
        if let Ok(ok_result) = &result {
            if let HttpOkResult::Content {
                content_type: _,
                content,
            } = ok_result
            {
                let scheme = get_scheme(&ctx);
                let host = get_host(&ctx);

                let mut placehloders = HashMap::new();

                placehloders.insert("SCHEME", scheme);

                placehloders.insert("HOST", host.to_string());
                let content = replace_placeholders(content, &placehloders);

                return Ok(HttpOkResult::Content {
                    content_type: Some(WebContentType::Json),
                    content,
                });
            }
        }
    }

    return result;
}

pub fn replace_placeholders(src: &[u8], placeholders: &HashMap<&str, String>) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::new();

    let mut i = 0;
    while i < src.len() {
        if src[i] == b'*' && src[i + 1] == b'*' && src[i + 2] == b'*' {
            let end_index = find_end_of_placeholder(src, i + 3).unwrap();

            let key = std::str::from_utf8(&src[i + 3..end_index]).unwrap();

            let value = placeholders.get(key);

            if let Some(value) = value {
                result.extend(value.as_bytes());
            }
            i = end_index + 2;
        } else {
            result.push(src[i]);
        }

        i += 1;
    }

    result
}

pub fn find_end_of_placeholder(src: &[u8], placeholder_start: usize) -> Option<usize> {
    for i in placeholder_start..src.len() {
        if src[i] == b'*' {
            return Some(i);
        }
    }

    None
}

fn get_scheme(ctx: &HttpContext) -> String {
    let headers = ctx.req.headers();
    let proto_header = headers.get("X-Forwarded-Proto");

    if let Some(scheme) = proto_header {
        let bytes = scheme.as_bytes();
        return std::str::from_utf8(bytes).unwrap().to_string();
    }

    let scheme = ctx.req.uri().scheme();

    match scheme {
        Some(scheme) => {
            return scheme.to_string();
        }
        None => "http".to_string(),
    }
}

fn get_host(ctx: &HttpContext) -> &str {
    std::str::from_utf8(&ctx.req.headers().get("host").unwrap().as_bytes()).unwrap()
}
