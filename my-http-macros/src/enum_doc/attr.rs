use proc_macro::TokenStream;

use crate::{
    enum_doc::enum_json::{EnumJson, HTTP_ENUM_ATTR_NAME},
    reflection::EnumCase,
};

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

    let doc = super::http_enum_structure::generate(name.as_str(), is_string, fields.as_slice());

    let fn_parse_str = super::fn_parse_str::generate(name.as_str(), fields.as_slice());

    let code = format!(
        r###" impl {name}{{
            pub fn {fn_name}()->{http_enum_structure}{{
                {doc}
            }}

            {fn_parse_str}
        }}
        "###,
        name = name,
        http_enum_structure = super::http_enum_structure::HTTP_ENUM_STRUCTURE,
        doc = doc,
        fn_name = crate::consts::FN_GET_HTTP_DATA_STRUCTURE
    );

    code.parse().unwrap()
}
