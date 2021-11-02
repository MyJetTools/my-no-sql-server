use hyper::{Body, Response};

use crate::{db_json_entity::DbEntityParseFail, json::JsonParseError};

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

impl From<JsonParseError> for HttpFailResult {
    fn from(value: JsonParseError) -> Self {
        Self {
            content_type: WebContentType::Text,
            status_code: 401,
            content: value.to_string().into_bytes(),
        }
    }
}

impl From<DbEntityParseFail> for HttpFailResult {
    fn from(src: DbEntityParseFail) -> Self {
        match src {
            DbEntityParseFail::FieldPartitionKeyIsRequired => Self {
                content_type: WebContentType::Text,
                status_code: 401,
                content: "PartitionKey field is required".as_bytes().to_vec(),
            },
            DbEntityParseFail::FieldRowKeyIsRequired => Self {
                content_type: WebContentType::Text,
                status_code: 401,
                content: "RowKey field is required".as_bytes().to_vec(),
            },

            DbEntityParseFail::JsonParseError(json_parse_error) => {
                HttpFailResult::from(json_parse_error)
            }
        }
    }
}
