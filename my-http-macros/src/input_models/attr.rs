use proc_macro::TokenStream;

use super::input_fields::InputFields;

pub fn impl_input_types(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;

    let fields = crate::reflection::StructProperty::read(ast);

    let fields = InputFields::new(fields);

    let doc = super::docs::generate_http_input(&fields);

    let struct_name = name.to_string();

    let model_reader = super::model_reader::generate(struct_name.as_str(), &fields);

    let code = format!(
        r###"impl {struct_name}{{
            pub fn get_input_params()->Vec<{http_input_parameter}>{{
                {doc}
            }}
            pub async fn parse_http_input(ctx:my_http_server::HttpContext)->Result<Self, my_http_server::HttpFailResult>{{
                {model_reader}
            }}
    }}"###,
        struct_name = struct_name,
        doc = doc,
        http_input_parameter = crate::types::HTTP_INPUT_PARAMETER_TYPE,
        model_reader = model_reader,
    );

    code.parse().unwrap()
}
