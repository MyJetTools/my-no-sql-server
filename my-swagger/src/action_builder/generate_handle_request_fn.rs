use super::attributes::AttributeModel;

pub fn generate_handle_request_fn(result: &mut String, attribute_model: &AttributeModel) {
    if let Some(attribute_model) = attribute_model.api_data.as_ref() {
        if let Some(input_data) = attribute_model.input_data.as_ref() {
            result.push_str(
                format!("let input_data = {input_data}::parse_http_input(ctx).await?;\n").as_str(),
            );
            result.push_str("handle_request(self, input_data, ctx).await");
        } else {
            result.push_str("handle_request(self, ctx).await");
        }
    } else {
        result.push_str("handle_request(self, ctx).await");
    }
}
