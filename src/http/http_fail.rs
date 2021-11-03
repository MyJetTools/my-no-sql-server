use hyper::{Body, Response};

use super::web_content_type::WebContentType;

#[derive(Debug)]
pub struct HttpFailResult {
    pub content_type: WebContentType,
    pub status_code: u16,
    pub content: Vec<u8>,
}

impl HttpFailResult {
    pub fn as_query_parameter_required(param_name: &str) -> Self {
        Self {
            content_type: WebContentType::Text,
            content: format!("Query parameter '{}' is required", param_name).into_bytes(),
            status_code: 301,
        }
    }

    pub fn as_not_found(text: String) -> Self {
        Self {
            content_type: WebContentType::Text,
            content: text.into_bytes(),
            status_code: 404,
        }
    }

    pub fn as_unauthorized() -> Self {
        Self {
            content_type: WebContentType::Text,
            content: "Unauthorized request".to_string().into_bytes(),
            status_code: 301,
        }
    }

    pub fn is_shutting_down() -> Self {
        Self {
            content_type: WebContentType::Text,
            content: "App is being shutting down".to_string().into_bytes(),
            status_code: 301,
        }
    }
}

impl Into<Response<Body>> for HttpFailResult {
    fn into(self) -> Response<Body> {
        Response::builder()
            .header("Content-Type", self.content_type.to_string())
            .status(self.status_code)
            .body(Body::from(self.content))
            .unwrap()
    }
}
