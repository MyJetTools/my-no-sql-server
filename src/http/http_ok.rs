use hyper::{Body, Response};
use my_http_utils::{HttpFailResult, WebContentType};
use serde::Serialize;

#[derive(Clone)]
pub enum HttpOkResult {
    Ok,

    Html {
        title: String,
        body: String,
    },
    Content {
        content_type: Option<WebContentType>,
        content: Vec<u8>,
    },
    Text {
        text: String,
    },
}

impl HttpOkResult {
    pub fn create_json_response<T: Serialize>(model: T) -> Result<HttpOkResult, HttpFailResult> {
        let json = serde_json::to_vec(&model).unwrap();
        Ok(HttpOkResult::Content {
            content_type: Some(WebContentType::Json),
            content: json,
        })
    }

    pub fn create_as_usize(number: usize) -> Result<HttpOkResult, HttpFailResult> {
        Ok(HttpOkResult::Content {
            content_type: Some(WebContentType::Text),
            content: number.to_string().into_bytes(),
        })
    }
}

impl Into<Response<Body>> for HttpOkResult {
    fn into(self) -> Response<Body> {
        return match self {
            HttpOkResult::Ok => Response::builder()
                .header("Content-Type", WebContentType::Text.to_string())
                .status(200)
                .body(Body::from("OK"))
                .unwrap(),
            HttpOkResult::Content {
                content_type,
                content,
            } => {
                if let Some(content_type) = content_type {
                    return Response::builder()
                        .header("Content-Type", content_type.to_string())
                        .status(200)
                        .body(Body::from(content))
                        .unwrap();
                } else {
                    let body = Body::from(content);
                    return Response::new(body);
                }
            }
            HttpOkResult::Text { text } => Response::builder()
                .header("Content-Type", WebContentType::Text.to_string())
                .status(200)
                .body(Body::from(text))
                .unwrap(),

            HttpOkResult::Html { title, body } => Response::builder()
                .header("Content-Type", "text/html")
                .status(200)
                .body(Body::from(compile_html(title, body)))
                .unwrap(),
        };
    }
}

fn compile_html(title: String, body: String) -> String {
    format!(
        r###"<html><head><title>{ver} MyNoSQLServer {title}</title>
        <link href="/css/bootstrap.css" rel="stylesheet" type="text/css" />
        </head><body>{body}</body></html>"###,
        ver = crate::app::APP_VERSION,
        title = title,
        body = body
    )
}
