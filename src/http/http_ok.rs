use std::sync::Arc;

use hyper::{Body, Response};
use my_http_utils::{HttpFailResult, WebContentType};
use serde::Serialize;

use crate::db::DbRow;

#[derive(Clone)]
pub enum HttpOkResult {
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
    DbRow(Arc<DbRow>),
    Empty,
}

impl HttpOkResult {
    pub fn create_json_response<T: Serialize>(model: T) -> Result<HttpOkResult, HttpFailResult> {
        let json = serde_json::to_vec(&model).unwrap();
        Ok(HttpOkResult::Content {
            content_type: Some(WebContentType::Json),
            content: json,
        })
    }

    pub fn as_usize(number: usize) -> Self {
        Self::Content {
            content_type: Some(WebContentType::Text),
            content: number.to_string().into_bytes(),
        }
    }

    pub fn as_db_row(db_row: Option<Arc<DbRow>>) -> Self {
        match db_row {
            Some(db_row) => Self::DbRow(db_row),
            None => Self::Empty,
        }
    }

    pub fn get_status_code(&self) -> u16 {
        match self {
            HttpOkResult::Html { title: _, body: _ } => 200,
            HttpOkResult::Content {
                content_type: _,
                content: _,
            } => 200,
            HttpOkResult::Text { text: _ } => 200,
            HttpOkResult::DbRow(_) => 200,
            HttpOkResult::Empty => 202,
        }
    }
}

impl Into<Response<Body>> for HttpOkResult {
    fn into(self) -> Response<Body> {
        let status_code = self.get_status_code();
        return match self {
            HttpOkResult::Content {
                content_type,
                content,
            } => {
                if let Some(content_type) = content_type {
                    return Response::builder()
                        .header("Content-Type", content_type.to_string())
                        .status(status_code)
                        .body(Body::from(content))
                        .unwrap();
                } else {
                    let body = Body::from(content);
                    return Response::new(body);
                }
            }
            HttpOkResult::Text { text } => Response::builder()
                .header("Content-Type", WebContentType::Text.to_string())
                .status(status_code)
                .body(Body::from(text))
                .unwrap(),

            HttpOkResult::Html { title, body } => Response::builder()
                .header("Content-Type", "text/html")
                .status(status_code)
                .body(Body::from(compile_html(title, body)))
                .unwrap(),
            HttpOkResult::DbRow(db_row) => {
                return Response::builder()
                    .header("Content-Type", WebContentType::Json.to_string())
                    .status(status_code)
                    .body(Body::from(db_row.data.clone()))
                    .unwrap();
            }
            HttpOkResult::Empty => {
                return Response::builder()
                    .header("Content-Type", WebContentType::Json.to_string())
                    .status(status_code)
                    .body(Body::empty())
                    .unwrap();
            }
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
