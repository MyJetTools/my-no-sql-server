use proc_macro::TokenStream;

use crate::reflection::StructProperty;

pub fn impl_output_types(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident.to_string();
    let fields = StructProperty::read(ast);
    let doc = super::docs::get_output_json_doc(name.as_str(), fields);

    let code = format!(
        r###" impl {name}{{
            pub fn get_doc()->{http_data_type}{{
                {doc}
            }}
        }}"###,
        name = name,
        http_data_type = crate::types::HTTP_DATA_TYPE,
        doc = doc
    );

    code.parse().unwrap()
}
