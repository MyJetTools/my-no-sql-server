use super::attributes::HttpResult;
use crate::consts::*;

pub fn generate(result: &mut String, results: &[HttpResult]) {
    result.push_str("vec![");

    for http_result in results {
        result.push_str(
            "my_http_server_controllers::controllers::documentation::out_results::HttpResult {",
        );
        result.push_str("nullable: false,");
        result.push_str(
            format!("description: \"{}\".to_string(),", http_result.description).as_str(),
        );

        result.push_str(format!("http_code: {},", http_result.status_code).as_str());

        if let Some(result_type) = &http_result.result_type {
            match result_type {
                super::attributes::ResultType::Object(object_name) => {
                    generate_as_object_or_array(object_name, result, "into_http_data_type_array");
                }
                super::attributes::ResultType::Array(object_name) => {
                    generate_as_object_or_array(object_name, result, "into_http_data_type_object");
                }
                super::attributes::ResultType::ArrayOfSimpleType(type_name) => {
                    result.push_str( format!("data_type: {HTTP_DATA_TYPE}::ArrayOf({HTTP_ARRAY_ELEMENT}::SimpleType({HTTP_SIMPLE_TYPE}::{type_name}))", ).as_str());
                }
                super::attributes::ResultType::SimpleType(type_name) => {
                    result.push_str(
                        format!("{HTTP_DATA_TYPE}::SimpleType({HTTP_SIMPLE_TYPE}::{type_name})",)
                            .as_str(),
                    );
                }
            }
        } else {
            result.push_str(format!("{HTTP_DATA_TYPE}::None",).as_str());
        }

        result.push_str("},");
    }

    result.push_str("]");
}

fn generate_as_object_or_array(object_name: &str, result: &mut String, into_structure: &str) {
    result.push_str(
        format!("data_type: {object_name}::get_http_data_structure().{into_structure}(),").as_str(),
    );
}
