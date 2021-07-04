pub fn clone_string_value(src: Option<&str>) -> Option<String> {
    let result = src?;
    let result = result.to_string();
    return Some(result);
}
