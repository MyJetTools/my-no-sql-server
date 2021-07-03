use hyper::{Body, Request};

use super::query_string::QueryString;

pub struct HttpContext {
    req: Request<Body>,
}

impl HttpContext {
    pub fn new(req: Request<Body>) -> Self {
        Self { req }
    }

    pub fn get_query_string(&self) -> QueryString {
        let query = self.req.uri().query();
        return QueryString::new(query);
    }

    pub async fn get_body(self) -> Vec<u8> {
        let body = self.req.into_body();
        let full_body = hyper::body::to_bytes(body).await.unwrap();

        let result = full_body.iter().cloned().collect::<Vec<u8>>();

        result
    }
}
