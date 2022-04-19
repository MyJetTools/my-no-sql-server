use crate::consts::*;
use proc_macro::TokenStream;

use super::attributes::AttributeModel;

pub fn build_action(attr: TokenStream, input: TokenStream) -> TokenStream {
    let mut result = input.to_string();

    let ast: syn::DeriveInput = syn::parse(input).unwrap();

    let attrs = AttributeModel::parse(attr);

    let struct_name = ast.ident.to_string();

    result.push_str("#[async_trait::async_trait]");

    result.push_str(
        format!(
            "impl {action_name} for {struct_name}{{",
            action_name = attrs.method.get_trait_name(),
        )
        .as_str(),
    );

    result.push_str("fn get_route(&self) -> &str {\"");
    result.push_str(attrs.route.as_str());
    result.push_str("\"}\n");

    result.push_str(
        format!("fn get_description(&self) -> Option<{HTTP_ACTION_DESCRIPTION}>{{").as_str(),
    );
    super::generate_http_action_description_fn(&mut result, attrs.api_data.as_ref());
    result.push_str("}");

    result.push_str(
        format!("async fn handle_request(&self, ctx: &mut {HTTP_CONTEXT}) -> Result<{HTTP_OK_RESULT}, {HTTP_FAIL_RESULT}> {{")
            .as_str(),
    );
    super::generate_handle_request_fn(&mut result, &attrs);
    result.push_str("}\n");

    result.push_str("}");

    result.parse().unwrap()
}
