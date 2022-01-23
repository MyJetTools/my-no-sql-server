use crate::reflection::StructProperty;

use super::out_json::OutputJson;

pub fn generate(name: &str, fields: Vec<StructProperty>) -> String {
    let json = OutputJson::new(fields);

    let mut result = String::new();

    result.push_str(format!("{} {{", crate::types::HTTP_OBJECT_STRUCTURE).as_str());
    result.push_str(format!("struct_id: \"{}\".to_string(),", name).as_str());

    result.push_str("fields: vec![");

    for field in json.fields {
        result.push_str(
            crate::types::compile_http_field(field.name(), &field.property.ty, true, None).as_str(),
        );
        result.push(',');
    }
    result.push_str("],}");

    result
}
