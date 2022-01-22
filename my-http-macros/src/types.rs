use crate::reflection::PropertyType;

pub const HTTP_INPUT_PARAMETER_TYPE: &str =
    "my_http_server::middlewares::controllers::documentation::in_parameters::HttpInputParameter";

const HTTP_FIELD_TYPE: &str =
    "my_http_server::middlewares::controllers::documentation::data_types::HttpField";

pub const HTTP_PARAMETER_INPUT_SRC: &str = "my_http_server::middlewares::controllers::documentation::in_parameters::HttpParameterInputSource";

pub const HTTP_DATA_TYPE: &str =
    "my_http_server::middlewares::controllers::documentation::data_types::HttpDataType";

pub const HTTP_OBJECT_STRUCTURE: &str =
    "my_http_server::middlewares::controllers::documentation::data_types::HttpObjectStructure";

pub fn compile_http_field(
    name: &str,
    pt: &PropertyType,
    required: bool,
    default: Option<&str>,
) -> String {
    let default = if let Some(default) = default {
        format!("Some(\"{}\".to_string())", default)
    } else {
        "None".to_string()
    };

    format!(
        "{http_field_type}::new(\"{name}\", {data_type}, {required}, {default})",
        http_field_type = HTTP_FIELD_TYPE,
        name = name,
        data_type = compile_data_type(pt, false),
        required = required,
        default = default
    )
}

pub fn compile_http_field_with_object(
    name: &str,
    body_type: &str,
    required: bool,
    default: Option<&str>,
) -> String {
    let default = if let Some(default) = default {
        format!("Some(\"{}\".to_string())", default)
    } else {
        "None".to_string()
    };

    format!(
        "{http_field_type}::new(\"{name}\", {data_type}, {required}, {default})",
        http_field_type = HTTP_FIELD_TYPE,
        name = name,
        data_type = format!("{}::get_doc()", body_type),
        required = required,
        default = default
    )
}

fn compile_data_type(pt: &PropertyType, inside_option: bool) -> String {
    if pt.is_option() {
        return compile_data_type(&pt.get_generic(), true);
    }

    if pt.raw == "String" {
        return format!("{}::as_string()", HTTP_DATA_TYPE);
    }

    if pt.raw == "u8" {
        return format!("{}::as_integer()", HTTP_DATA_TYPE);
    }

    if pt.raw == "i8" {
        return format!("{}::as_integer()", HTTP_DATA_TYPE);
    }

    if pt.raw == "u16" {
        return format!("{}::as_integer()", HTTP_DATA_TYPE);
    }

    if pt.raw == "i16" {
        return format!("{}::as_integer()", HTTP_DATA_TYPE);
    }

    if pt.raw == "u32" {
        return format!("{}::as_integer()", HTTP_DATA_TYPE);
    }

    if pt.raw == "i32" {
        return format!("{}::as_integer()", HTTP_DATA_TYPE);
    }

    if pt.raw == "u64" {
        return format!("{}::as_long()", HTTP_DATA_TYPE);
    }

    if pt.raw == "i64" {
        return format!("{}::as_long()", HTTP_DATA_TYPE);
    }

    if pt.raw == "usize" {
        return format!("{}::as_long()", HTTP_DATA_TYPE);
    }

    if pt.raw == "bool" {
        return format!("{}::as_bool()", HTTP_DATA_TYPE);
    }

    if pt.raw == "isize" {
        return format!("{}::as_long()", HTTP_DATA_TYPE);
    }

    if pt.raw == "Vec" {
        return format!("{}::None", HTTP_DATA_TYPE);
    }

    if inside_option {
        panic!("Not supported type: Option<{}>", pt.raw);
    } else {
        return format!("{}::get_doc()", pt.raw);
    }
}
