pub fn encode(partition_name: &str) -> String {
    base64::encode(partition_name.as_bytes())
}

pub fn decode(file_name: &str) -> String {
    let partition_name = base64::decode(file_name).unwrap();
    String::from_utf8(partition_name).unwrap()
}
