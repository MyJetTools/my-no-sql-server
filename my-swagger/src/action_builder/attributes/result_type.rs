pub enum ResultType {
    Object(String),
    Array(String),
    ArrayOfSimpleType(String),
    SimpleType(String),
}

impl ResultType {
    pub fn new(model: Option<String>, model_as_array: Option<String>) -> Option<Self> {
        if let Some(model_as_object) = model {
            if is_simple_type(model_as_object.as_str()) {
                return ResultType::SimpleType(model_as_object).into();
            }

            return ResultType::Object(model_as_object).into();
        }

        if let Some(model_as_array) = model_as_array {
            if is_simple_type(model_as_array.as_str()) {
                return ResultType::ArrayOfSimpleType(model_as_array).into();
            }

            return ResultType::Array(model_as_array).into();
        }

        None
    }
}

fn is_simple_type(src: &str) -> bool {
    match src {
        "String" => true,
        "Integer" => true,
        "Long" => true,
        "Float" => true,
        "Double" => true,
        "Byte" => true,
        "Binary" => true,
        "Boolean" => true,
        "Date" => true,
        "DateTime" => true,
        "Password" => true,
        _ => false,
    }
}
