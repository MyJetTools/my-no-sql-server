use proc_macro::TokenStream;

use crate::{
    enum_doc::enum_json::{EnumJson, HTTP_ENUM_ATTR_NAME},
    reflection::EnumCase,
};

const HTTP_ENUM_STRUCTURE: &str =
    "my_http_server::middlewares::controllers::documentation::data_types::HttpEnumStructure";
const ENUM_TYPE: &str =
    "my_http_server::middlewares::controllers::documentation::data_types::EnumType";
const HTTP_ENUM_CASE: &str =
    "my_http_server::middlewares::controllers::documentation::data_types::HttpEnumCase";

pub fn impl_enum_doc(ast: &syn::DeriveInput, is_string: bool) -> TokenStream {
    let name = &ast.ident.to_string();
    let src_fields = EnumCase::read(ast);

    let mut fields = Vec::new();

    for src_field in src_fields {
        let name = src_field.name.to_string();
        if let Some(enum_json) = EnumJson::new(src_field) {
            fields.push(enum_json);
        } else {
            panic!(
                "Enum case {} does not have #[{}] attribute",
                name, HTTP_ENUM_ATTR_NAME
            )
        }
    }

    let doc = generate_doc(name.as_str(), is_string, fields.as_slice());

    let from_str = super::impl_from_str::generate(name.as_str(), fields.as_slice());

    let code = format!(
        r###" impl {name}{{
            pub fn get_doc()->{http_data_type}{{
                {doc}
            }}
        }}
        {from_str}"###,
        name = name,
        http_data_type = crate::types::HTTP_DATA_TYPE,
        doc = doc,
        from_str = from_str
    );

    code.parse().unwrap()
}

fn generate_doc(name: &str, is_string: bool, enum_cases: &[EnumJson]) -> String {
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

    format!("{}::Enum({})", crate::types::HTTP_DATA_TYPE, result)
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
