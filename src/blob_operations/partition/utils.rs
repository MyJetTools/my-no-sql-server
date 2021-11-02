pub fn get_blob_file_name_by_partition_name(partition_name: &str) -> String {
    base64::encode(partition_name.as_bytes())
}
