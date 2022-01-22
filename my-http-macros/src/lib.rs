extern crate proc_macro;
use proc_macro::TokenStream;

use syn;

mod enum_doc;
mod input_models;
mod output_models;
mod reflection;
mod types;

#[proc_macro_derive(
    MyHttpInput,
    attributes(http_query, http_header, http_body, http_form, http_body_type)
)]
pub fn my_http_input_doc_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    crate::input_models::attr::impl_input_types(&ast)
}

#[proc_macro_derive(MyHttpDocument)]
pub fn my_http_input_process_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    crate::output_models::attr::impl_output_types(&ast)
}

#[proc_macro_derive(MyHttpStringEnum, attributes(http_enum_case))]
pub fn my_http_string_enum_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    crate::enum_doc::attr::impl_enum_doc(&ast, true)
}

#[proc_macro_derive(MyHttpIntegerEnum, attributes(http_enum_case))]
pub fn my_http_integer_enum_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    crate::enum_doc::attr::impl_enum_doc(&ast, false)
}
