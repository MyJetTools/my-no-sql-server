use crate::{input_models::input_fields::InputField, reflection::PropertyType};

pub fn get_source_to_read<'s>(form_data: bool) -> &'s str {
    if form_data {
        "form_data"
    } else {
        "query_string"
    }
}

pub fn read_string_parameter_with_default_value(
    form_data: bool,
    input_field: &InputField,
    default: &str,
) -> String {
    let optional_string = generate_read_optional_string_parameter(form_data, input_field);
    let get_value = option_of_str_to_default(optional_string.as_str(), default);
    compile_read_line(input_field, get_value.as_str())
}

pub fn read_system_parameter_with_default_value(
    form_data: bool,
    input_field: &InputField,
    default: &str,
) -> String {
    let optional_string = generate_read_optional_parameter(form_data, input_field);
    let get_value = option_to_system_default(optional_string.as_str(), default);
    compile_read_line(input_field, get_value.as_str())
}

pub fn read_parameter_with_default_value(
    form_data: bool,
    input_field: &InputField,
    default: &str,
) -> String {
    let optional_string = generate_read_optional_string_parameter(form_data, input_field);
    let get_value = option_to_default(optional_string.as_str(), default, &input_field.property.ty);
    compile_read_line(input_field, get_value.as_str())
}

pub fn read_optional_string_parameter(form_data: bool, input_field: &InputField) -> String {
    let src = get_source_to_read(form_data);
    let get_optional_value = format!(
        "{src}.get_optional_string_parameter(\"{http_name}\")",
        src = src,
        http_name = input_field.name()
    );

    let get_value = option_of_str_to_option_of_string(get_optional_value.as_str());
    compile_read_line(input_field, get_value.as_str())
}

pub fn read_optional_parameter(form_data: bool, input_field: &InputField) -> String {
    let src = get_source_to_read(form_data);

    let get_value = format!(
        "{src}.get_optional_parameter(\"{http_name}\")",
        src = src,
        http_name = input_field.name()
    );

    compile_read_line(input_field, get_value.as_str())
}

pub fn read_from_headers(input_field: &InputField) -> String {
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

fn compile_read_line(input_field: &InputField, reading_line: &str) -> String {
    format!(
        "{struct_field_name}: {reading_line},\n",
        struct_field_name = input_field.struct_field_name(),
        reading_line = reading_line
    )
}

fn generate_read_optional_string_parameter(form_data: bool, input_field: &InputField) -> String {
    let src = get_source_to_read(form_data);

    format!(
        "{src}.get_optional_string_parameter(\"{http_name}\")",
        src = src,
        http_name = input_field.name()
    )
}

fn generate_read_optional_parameter(form_data: bool, input_field: &InputField) -> String {
    let src = get_source_to_read(form_data);

    format!(
        "{src}.get_optional_parameter(\"{http_name}\")",
        src = src,
        http_name = input_field.name()
    )
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

fn option_of_str_to_default(expr: &str, default: &str) -> String {
    format!(
        r###"
        if let Some(value) = {expr}{{
            value.to_string()
        }}else{{
            "{default}".to_string()
        }}
    "###,
        expr = expr,
        default = default
    )
}

fn option_to_system_default(expr: &str, default: &str) -> String {
    format!(
        r###"
        if let Some(value) = {expr}{{
            value
        }}else{{
            {default}
        }}
    "###,
        expr = expr,
        default = default
    )
}

fn option_to_default(expr: &str, default: &str, ty: &PropertyType) -> String {
    format!(
        r###"
        if let Some(value) = {expr}{{
            {ty}::from_str(value)?
        }}else{{
            {ty}::from_str("{default}")?
        }}
    "###,
        expr = expr,
        default = default,
        ty = ty.type_name
    )
}
