use my_http_server::{HttpFailResult, HttpRequest};

pub trait StandardParamsReader {
    fn get_api_key(&self) -> Result<&str, HttpFailResult>;
}

impl StandardParamsReader for HttpRequest {
    fn get_api_key(&self) -> Result<&str, HttpFailResult> {
        return self.get_required_header("apikey");
    }
}
