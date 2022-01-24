use super::enum_json::EnumJson;

const HTTP_FAIL_RESULT: &str = "my_http_server::HttpFailResult";

pub fn generate(name: &str, enum_cases: &[EnumJson]) -> String {
    format!(
        r###"pub fn {fn_parse_str}(src: &str) -> Result<Self, {http_fail_result}> {{
                {content}
            }}"###,
        content = generate_content(name, enum_cases),
        http_fail_result = HTTP_FAIL_RESULT,
        fn_parse_str = crate::consts::FN_PARSE_STR
    )
}

fn generate_content(name: &str, enum_cases: &[EnumJson]) -> String {
    let mut result = String::new();
    for enum_case in enum_cases {
        let line_to_add = format!(
            "if src == \"{value}\" || src == \"{the_id}\"{{return Ok(Self::{value})}}\n",
            value = enum_case.value(),
            the_id = enum_case.id()
        );

        result.push_str(line_to_add.as_str());
    }

    let line_to_add = format!(
        "Err({http_fail_result}::as_forbidden(Some(\"{err}\".to_string())))",
        http_fail_result = HTTP_FAIL_RESULT,
        err = format!("Can not parse {} enum", name)
    );

    result.push_str(line_to_add.as_str());

    result
}
