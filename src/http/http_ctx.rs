use hyper::{Body, Request};
use my_http_utils::{HttpFailResult, QueryString};

pub struct HttpContext {
    pub req: Request<Body>,
}

impl HttpContext {
    pub fn new(req: Request<Body>) -> Self {
        Self { req }
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
