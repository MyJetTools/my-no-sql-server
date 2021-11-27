use my_http_utils::HttpFailResult;

use super::http_ctx::HttpContext;

pub trait StandardParamsReader {
    fn get_api_key(&self) -> Result<&str, HttpFailResult>;
}

impl StandardParamsReader for HttpContext {
    fn get_api_key(&self) -> Result<&str, HttpFailResult> {
        return self.get_required_header("apikey");
    }
}
