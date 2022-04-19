pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
}

impl HttpMethod {
    pub fn parse(src: &str) -> Self {
        match src {
            "GET" => HttpMethod::Get,
            "POST" => HttpMethod::Post,
            "PUT" => HttpMethod::Put,
            "DELETE" => HttpMethod::Delete,
            _ => panic!("Unsupported http method: {}", src),
        }
    }
    pub fn get_trait_name(&self) -> &str {
        match self {
            HttpMethod::Get => "my_http_server_controllers::controllers::actions::GetAction",
            HttpMethod::Post => "my_http_server_controllers::controllers::actions::PostAction",
            HttpMethod::Put => "my_http_server_controllers::controllers::actions::PutAction",
            HttpMethod::Delete => "my_http_server_controllers::controllers::actions::DeleteAction",
        }
    }
}
