use my_http_server_controllers::controllers::documentation::{
    data_types::{HttpDataType, HttpObjectStructure},
    out_results::HttpResult,
};

pub fn empty(description: &str) -> HttpResult {
    HttpResult {
        http_code: 202,
        nullable: true,
        description: description.to_string(),
        data_type: HttpDataType::None,
    }
}

pub fn table_not_found() -> HttpResult {
    HttpResult {
        http_code: 400,
        nullable: true,
        description: "Table not found".to_string(),
        data_type: HttpDataType::Object(HttpObjectStructure::new("EmptyContract")),
    }
}
