use super::input_fields::{InputField, InputFieldSource, InputFields};

pub fn generate_http_input(fields: &InputFields) -> String {
    let mut result = String::new();

    for input_field in &fields.fields {
        let itm = generate_http_input_parameter(input_field);
        result.push_str(itm.as_str());
    }

    format!("vec![{}]", result)
}

fn generate_http_input_parameter(input_field: &InputField) -> String {
    let http_field = if input_field.is_body() {
        if let Some(body_type) = input_field.my_attr.get_value("body_type") {
            crate::types::compile_http_field_with_object(
                input_field.name(),
                body_type,
                input_field.required(),
                input_field.default(),
            )
        } else {
            crate::types::compile_http_field(
                input_field.name(),
                &input_field.property.ty,
                input_field.required(),
                input_field.default(),
            )
        }
    } else {
        crate::types::compile_http_field(
            input_field.name(),
            &input_field.property.ty,
            input_field.required(),
            input_field.default(),
        )
    };

    format!(
        r###"{http_input_parameter_type}{{
                    field: {http_field},
                    description: "{description}".to_string(),
                    source: {source},
                }},"###,
        http_input_parameter_type = crate::types::HTTP_INPUT_PARAMETER_TYPE,
        http_field = http_field,
        description = input_field.description(),
        source = get_input_src(input_field)
    )
}

fn get_input_src(field: &InputField) -> String {
    let field = match field.src {
        InputFieldSource::Query => "Query",
        InputFieldSource::Path => "Path",
        InputFieldSource::Header => "Header",
        InputFieldSource::Body => "Body",
        InputFieldSource::Form => "Form",
    };

    return format!("{}::{}", crate::types::HTTP_PARAMETER_INPUT_SRC, field);
}
