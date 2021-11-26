use hyper::{Body, Request};
use my_http_utils::{HttpFailResult, QueryString};

pub struct HttpContext {
    pub req: Request<Body>,
}

impl HttpContext {
    pub fn new(req: Request<Body>) -> Self {
        Self { req }
    }

    pub fn get_required_header(&self, header_name: &str) -> Result<&str, HttpFailResult> {
        for (name, value) in self.req.headers() {
            if name.as_str() == header_name {
                return Ok(value.to_str().unwrap());
            }
        }

        return Err(HttpFailResult::as_query_parameter_required(
            format!("Header: {}", header_name).as_str(),
        ));
    }

    pub fn get_query_string(&self) -> Result<QueryString, HttpFailResult> {
        let query = self.req.uri().query();
        match query {
            Some(query) => Ok(QueryString::new(query)?),
            None => panic!("Can not get query"),
        }
    }

    pub async fn get_body(self) -> Vec<u8> {
        let body = self.req.into_body();
        let full_body = hyper::body::to_bytes(body).await.unwrap();

        let result = full_body.iter().cloned().collect::<Vec<u8>>();

        result
    }
}
