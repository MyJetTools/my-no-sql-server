use crate::input_models::input_fields::{InputField, InputFieldSource, InputFields};

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
                let line_to_add = super::rust_builders::read_from_headers(input_field);
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
    if let Some(default) = input_field.default() {
        if input_field.property.ty.is_option() {
            panic!("It does not make sence to have default value and Option type");
        }

        return read_with_default(form_data, input_field, default);
    }

    if input_field.required() {
        return read_required(form_data, input_field);
    } else {
        let type_of_option = input_field.property.ty.get_generic();
        if type_of_option.is_string() {
            return super::rust_builders::read_optional_string_parameter(form_data, input_field);
        } else {
            return super::rust_builders::read_optional_parameter(form_data, input_field);
        }
    }
}

fn read_with_default(form_data: bool, input_field: &InputField, default: &str) -> String {
    if input_field.property.ty.is_string() {
        return super::rust_builders::read_string_parameter_with_default_value(
            form_data,
            input_field,
            default,
        );
    }
    if input_field.property.ty.is_system_type() {
        return super::rust_builders::read_system_parameter_with_default_value(
            form_data,
            input_field,
            default,
        );
    } else {
        return super::rust_builders::read_parameter_with_default_value(
            form_data,
            input_field,
            default,
        );
    }
}

fn read_required(form_data: bool, input_field: &InputField) -> String {
    let src = super::rust_builders::get_source_to_read(form_data);
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
}
