use super::enum_json::EnumJson;

pub const HTTP_ENUM_STRUCTURE: &str =
    "my_http_server::middlewares::controllers::documentation::data_types::HttpEnumStructure";
const ENUM_TYPE: &str =
    "my_http_server::middlewares::controllers::documentation::data_types::EnumType";
const HTTP_ENUM_CASE: &str =
    "my_http_server::middlewares::controllers::documentation::data_types::HttpEnumCase";

pub fn generate(name: &str, is_string: bool, enum_cases: &[EnumJson]) -> String {
    let mut result = String::new();

    result.push_str(format!("{} {{", HTTP_ENUM_STRUCTURE).as_str());
    result.push_str(format!("struct_id: \"{}\".to_string(),", name).as_str());

    let tp = if is_string { "String" } else { "Integer" };

    result.push_str(
        format!(
            "enum_type: {enum_type}::{tp},",
            enum_type = ENUM_TYPE,
            tp = tp
        )
        .as_str(),
    );

    result.push_str("cases: vec![");

    for enum_json in enum_cases {
        if let Some(data_to_add) = compile_enum_case(enum_json) {
            result.push_str(data_to_add.as_str());
            result.push(',');
        } else {
        }
    }
    result.push_str("],}");

    result
}

fn compile_enum_case(enum_case: &EnumJson) -> Option<String> {
    format!(
        "{tp}{{id:{the_id}, value:\"{value}\".to_string(), description:\"{description}\".to_string()}}",
        tp = HTTP_ENUM_CASE,
        the_id = enum_case.id(),
        value = enum_case.value(),
        description = enum_case.description()
    )
    .into()
}
