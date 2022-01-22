use crate::input_models::input_fields::InputFieldSource;

use super::input_fields::{InputField, InputFields};

pub fn generate(name: &str, input_fields: &InputFields) -> String {
    let mut result = String::new();

    if input_fields.has_query() {
        result.push_str("let query_string = ctx.get_query_string()?;\n");
    }

    if input_fields.has_form_data() {
        result.push_str("let form_data = ctx.get_form_data()?;\n");
    }

    result.push_str("Ok(");
    result.push_str(name);
    result.push('{');

    for input_field in &input_fields.fields {
        match &input_field.src {
            InputFieldSource::Query => {
                let line_to_add = build_reading(input_field, false);
                result.push_str(line_to_add.as_str());
            }
            InputFieldSource::Path => {
                let line_to_add = if input_field.required() {
                    format!(
                        "{}: ctx.get_value_from_path(\"{}\")?.to_string(),",
                        input_field.struct_field_name(),
                        input_field.name()
                    )
                } else {
                    format!(
                        "{}: ctx.get_value_from_path_optional(\"{}\")?,",
                        input_field.struct_field_name(),
                        input_field.name()
                    )
                };

                result.push_str(line_to_add.as_str());
            }
            InputFieldSource::Header => {
                let line_to_add = read_from_headers(input_field);
                result.push_str(line_to_add.as_str());
            }
            InputFieldSource::Body => {}
            InputFieldSource::Form => {
                let line_to_add = build_reading(input_field, true);
                result.push_str(line_to_add.as_str());
            }
        }
    }

    if let Some(body_field) = input_fields.get_body_field() {
        let line_to_add = format!(
            "{}: ctx.get_body_raw().await?,",
            body_field.struct_field_name()
        );
        result.push_str(line_to_add.as_str());
    }

    result.push_str("})");

    result
}

fn build_reading(input_field: &InputField, form_data: bool) -> String {
    let src = if form_data {
        "form_data"
    } else {
        "query_string"
    };

    if input_field.required() {
        if input_field.property.ty.is_string() {
            format!(
                "{struct_field_name}: {src}.get_required_string_parameter(\"{http_name}\")?.to_string(),\n",
                struct_field_name = input_field.struct_field_name(),
                src = src,
                http_name = input_field.name()
            )
        } else {
            format!(
                "{struct_field_name}: {src}.get_required_parameter(\"{http_name}\")?,\n",
                struct_field_name = input_field.struct_field_name(),
                src = src,
                http_name = input_field.name()
            )
        }
    } else {
        let type_of_option = input_field.property.ty.get_generic();
        if type_of_option.is_string() {
            let get_optional_value = format!(
                "{src}.get_optional_string_parameter(\"{http_name}\")",
                src = src,
                http_name = input_field.name()
            );

            format!(
                "{struct_field_name}: {get_optional_value},\n",
                struct_field_name = input_field.struct_field_name(),
                get_optional_value = option_of_str_to_option_of_string(get_optional_value.as_str()),
            )
        } else {
            format!(
                "{struct_field_name}: {src}.get_optional_parameter(\"{http_name}\"),\n",
                struct_field_name = input_field.struct_field_name(),
                src = src,
                http_name = input_field.name()
            )
        }
    }
}

fn read_from_headers(input_field: &InputField) -> String {
    if input_field.required() {
        if input_field.property.ty.is_string() {
            format!(
                "{struct_field_name}: ctx.get_required_header(\"{http_name}\")?.to_string(),\n",
                struct_field_name = input_field.struct_field_name(),
                http_name = input_field.name()
            )
        } else {
            panic!("Header can only be read to String typed property");
        }
    } else {
        if input_field.property.ty.get_generic().is_string() {
            let get_optional_header = format!(
                "ctx.get_optional_header(\"{http_name}\")",
                http_name = input_field.name()
            );

            format!(
                "{struct_field_name}: {str_converions},\n",
                struct_field_name = input_field.struct_field_name(),
                str_converions = option_of_str_to_option_of_string(get_optional_header.as_str())
            )
        } else {
            panic!("Header can only be read to String typed property");
        }
    }
}

fn option_of_str_to_option_of_string(expr: &str) -> String {
    format!(
        r###"
        if let Some(value) = {expr}{{
            Some(value.to_string())
        }}else{{
            None
        }}
    "###,
        expr = expr,
    )
}
