use proc_macro::TokenStream;

use super::{http_method::HttpMethod, HttpResult};

pub struct ApiData {
    pub controller: String,
    pub description: String,
    pub input_data: Option<String>,
    pub result: Vec<HttpResult>,
}

impl ApiData {
    pub fn new(
        controller: Option<String>,
        description: Option<String>,
        input_data: Option<String>,
        result: Vec<HttpResult>,
    ) -> Option<Self> {
        if controller.is_none() {
            return None;
        }

        if description.is_none() {
            panic!("description is not found");
        }

        Self {
            controller: controller.unwrap(),
            description: description.unwrap(),
            input_data,
            result,
        }
        .into()
    }
}

pub struct AttributeModel {
    pub method: HttpMethod,
    pub route: String,
    pub api_data: Option<ApiData>,
}

impl AttributeModel {
    pub fn parse(attr: TokenStream) -> Self {
        let str = attr.to_string().into_bytes();

        let mut bytes = str.as_slice();

        let mut method: Option<String> = None;
        let mut route: Option<String> = None;

        let mut controller: Option<String> = None;
        let mut description: Option<String> = None;
        let mut input_data: Option<String> = None;
        let mut result: Option<String> = None;

        loop {
            let separator_pos = find(bytes, ':' as u8);

            if separator_pos.is_none() {
                break;
            }

            let separator_pos = separator_pos.unwrap();

            let key = std::str::from_utf8(&bytes[..separator_pos]).unwrap().trim();

            //println!("Key: [{}]", key);

            bytes = &bytes[separator_pos..];

            let start_value_pos = find_one_of_these(bytes, &['[' as u8, '"' as u8]);

            if start_value_pos.is_none() {
                break;
            }

            let start_value_pos = start_value_pos.unwrap();

            bytes = &bytes[start_value_pos..];

            let end_byte = if bytes[0] == '[' as u8 {
                ']' as u8
            } else {
                bytes[0]
            };

            bytes = &bytes[1..];

            let end_value_pos = find(bytes, end_byte);

            if end_value_pos.is_none() {
                break;
            }

            let end_value_pos = end_value_pos.unwrap();

            let value = std::str::from_utf8(&bytes[..end_value_pos]).unwrap();

            //println!("Value: [{}]", value);

            match key {
                "method" => {
                    method = Some(value.to_string());
                }
                "controller" => {
                    controller = Some(value.to_string());
                }
                "route" => {
                    route = Some(value.to_string());
                }
                "description" => {
                    description = Some(value.to_string());
                }
                "input_data" => {
                    input_data = Some(value.to_string());
                }

                "result" => {
                    result = Some(value.to_string());
                }

                _ => {}
            }

            bytes = &bytes[end_value_pos..];

            let separator_pos = find(bytes, ',' as u8);

            if separator_pos.is_none() {
                break;
            }

            let separator_pos = separator_pos.unwrap();
            bytes = &bytes[separator_pos + 1..];
        }

        if method.is_none() {
            panic!("[method] is not found");
        }

        if route.is_none() {
            panic!("[route] is not found");
        }

        Self {
            method: HttpMethod::parse(method.as_ref().unwrap()),
            route: route.unwrap(),
            api_data: ApiData::new(controller, description, input_data, HttpResult::new(result)),
        }
    }
}

pub fn find(src: &[u8], symbol: u8) -> Option<usize> {
    for i in 0..src.len() {
        if src[i] == symbol {
            return Some(i);
        }
    }

    None
}

pub fn find_one_of_these(src: &[u8], symbols: &[u8]) -> Option<usize> {
    for i in 0..src.len() {
        for s in symbols {
            if src[i] == *s {
                return Some(i);
            }
        }
    }

    None
}
