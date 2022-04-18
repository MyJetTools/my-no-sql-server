use crate::consts::*;

use super::attributes::ApiData;

pub fn generate_http_action_description_fn(result: &mut String, api_data: Option<&ApiData>) {
    if api_data.is_none() {
        result.push_str("None");
        return;
    }

    let api_data = api_data.unwrap();

    result.push_str(HTTP_ACTION_DESCRIPTION);
    result.push_str("{");

    result.push_str("controller_name: \"");
    result.push_str(api_data.controller.as_str());
    result.push('"');
    result.push(',');

    result.push_str("description: \"");
    result.push_str(api_data.description.as_str());
    result.push('"');
    result.push(',');

    result.push_str("input_params: ");
    generate_get_input_params(result, api_data);
    result.push(',');

    result.push_str("results: ");
    super::result_model_generator::generate(result, &api_data.result);
    result.push_str("}.into()");
}

fn generate_get_input_params(result: &mut String, api_data: &ApiData) {
    if let Some(input_data) = api_data.input_data.as_ref() {
        result.push_str(input_data);
        result.push_str("::get_input_params().into()");
    } else {
        result.push_str("None");
    }
}
