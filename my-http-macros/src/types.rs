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
        data_type = format!(
            "{}::{fn_name}().into_http_data_type_object()",
            body_type,
            fn_name = crate::consts::FN_GET_HTTP_DATA_STRUCTURE
        ),
        required = required,
        default = default,
    )
}

fn compile_data_type(pt: &PropertyType, inside_option: bool) -> String {
    if pt.is_option() {
        return compile_data_type(&pt.get_generic(), true);
    }

    if pt.type_name == "String" {
        return format!("{}::as_string()", HTTP_DATA_TYPE);
    }

    if pt.type_name == "u8" {
        return format!("{}::as_integer()", HTTP_DATA_TYPE);
    }

    if pt.type_name == "i8" {
        return format!("{}::as_integer()", HTTP_DATA_TYPE);
    }

    if pt.type_name == "u16" {
        return format!("{}::as_integer()", HTTP_DATA_TYPE);
    }

    if pt.type_name == "i16" {
        return format!("{}::as_integer()", HTTP_DATA_TYPE);
    }

    if pt.type_name == "u32" {
        return format!("{}::as_integer()", HTTP_DATA_TYPE);
    }

    if pt.type_name == "i32" {
        return format!("{}::as_integer()", HTTP_DATA_TYPE);
    }

    if pt.type_name == "u64" {
        return format!("{}::as_long()", HTTP_DATA_TYPE);
    }

    if pt.type_name == "i64" {
        return format!("{}::as_long()", HTTP_DATA_TYPE);
    }

    if pt.type_name == "usize" {
        return format!("{}::as_long()", HTTP_DATA_TYPE);
    }

    if pt.type_name == "bool" {
        return format!("{}::as_bool()", HTTP_DATA_TYPE);
    }

    if pt.type_name == "isize" {
        return format!("{}::as_long()", HTTP_DATA_TYPE);
    }

    if pt.type_name == "Vec" {
        return format!("{}::None", HTTP_DATA_TYPE);
    }

    if inside_option {
        panic!("Not supported type: Option<{}>", pt.type_name);
    } else {
        return format!(
            "{}::{}().into_http_data_type_object()",
            pt.type_name,
            func_name = crate::consts::FN_GET_HTTP_DATA_STRUCTURE
        );
    }
}
