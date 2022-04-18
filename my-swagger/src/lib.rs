extern crate proc_macro;

use proc_macro::TokenStream;
mod action_builder;
mod consts;

#[proc_macro_attribute]
pub fn http_route(attr: TokenStream, item: TokenStream) -> TokenStream {
    crate::action_builder::build_action(attr, item)
}
