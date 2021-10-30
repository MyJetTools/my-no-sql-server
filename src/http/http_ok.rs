use std::sync::Arc;

use hyper::{Body, Response};
use serde::Serialize;

use crate::db::{DbOperationResult, DbRow};

use super::{http_fail::HttpFailResult, web_content_type::WebContentType};

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

impl Into<HttpOkResult> for DbOperationResult {
    fn into(self) -> HttpOkResult {
        match self {
            DbOperationResult::Rows { rows } => HttpOkResult::Content {
                content_type: Some(WebContentType::Json),
                content: to_json_array(rows),
            },
            DbOperationResult::Row { row } => HttpOkResult::Content {
                content_type: Some(WebContentType::Json),
                content: row.data.clone(),
            },
        }
    }
}

pub fn to_json_array(db_rows: Option<Vec<Arc<DbRow>>>) -> Vec<u8> {
    if db_rows.is_none() {
        return vec![
            crate::json::consts::OPEN_ARRAY,
            crate::json::consts::CLOSE_ARRAY,
        ];
    }

    let mut json = Vec::new();

    let db_rows = db_rows.unwrap();

    for db_row in db_rows.as_slice() {
        if json.len() == 0 {
            json.push(crate::json::consts::OPEN_ARRAY);
        } else {
            json.push(crate::json::consts::COMMA);
        }

        json.extend(db_row.data.as_slice());
    }

    json.push(crate::json::consts::CLOSE_ARRAY);

    return json;
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
