use super::utils::JsonTimeStamp;

pub struct TimeStampValuePosition {
    pub start: usize,
    pub end: usize,
}

pub fn replace_timestamp_value(
    raw: &[u8],
    time_stamp_value_position: &TimeStampValuePosition,
    json_time_stamp: &JsonTimeStamp,
) -> Vec<u8> {
    let timestamp_value = format!("{dq}{val}{dq}", dq = '"', val = json_time_stamp.as_str());

    let timestamp_value = timestamp_value.as_bytes();

    let new_len = timestamp_value.len()
        + (raw.len() - (time_stamp_value_position.end - time_stamp_value_position.start));

    let mut result = Vec::with_capacity(new_len);

    let before = &raw[..time_stamp_value_position.start];

    result.extend_from_slice(before);

    result.extend_from_slice(timestamp_value);

    let after = &raw[time_stamp_value_position.end..];
    result.extend_from_slice(after);

    return result;
}

pub fn inject(raw: &[u8], time_stamp: &JsonTimeStamp) -> Vec<u8> {
    let date_time = format!(
        ",{dq}{ts}{dq}:{dq}{val}{dq}",
        dq = '"',
        ts = super::consts::TIME_STAMP,
        val = time_stamp.as_str(),
    );

    let date_time = date_time.as_bytes();

    let end_of_json = get_the_end_of_the_json(raw);

    let mut result = Vec::with_capacity(raw.len() + date_time.len());

    result.extend_from_slice(&raw[..end_of_json]);

    result.extend_from_slice(date_time);

    result.extend_from_slice(&raw[end_of_json..]);
    result
}

fn get_the_end_of_the_json(data: &[u8]) -> usize {
    for i in (0..data.len()).rev() {
        if data[i] == crate::json::consts::CLOSE_BRACKET {
            return i;
        }
    }

    panic!("Invalid Json. Can not find the end of json");
}

#[cfg(test)]
mod tests {

    use crate::db_json_entity::{date_time_injector::TimeStampValuePosition, utils::JsonTimeStamp};

    #[test]
    fn test_timestamp_injection() {
        let json_ts = JsonTimeStamp::now();

        let src_json = r#"{"Field1":"Value1"} "#;

        let dest_json_etalon = format!(
            "{}\"Field1\":\"Value1\",\"TimeStamp\":\"{}\"{} ",
            '{',
            json_ts.as_str(),
            '}'
        );

        let result = super::inject(src_json.as_bytes(), &json_ts);

        let dest_json = String::from_utf8(result).unwrap();

        println!("{}", dest_json_etalon);
        println!("{}", dest_json);

        assert_eq!(dest_json_etalon, dest_json);
    }

    #[test]
    fn test_replace_null_to_timestamp() {
        let json_ts = JsonTimeStamp::now();

        let src_json = r#"{"Field1":"Value1","TimeStamp":null}"#;

        let dest_json_etalon = format!(
            "{}\"Field1\":\"Value1\",\"TimeStamp\":\"{}\"{}",
            '{',
            json_ts.as_str(),
            '}'
        );

        let index = src_json.find("null").unwrap();

        let ts_value_position = TimeStampValuePosition {
            start: index,
            end: index + 4,
        };

        let result =
            super::replace_timestamp_value(src_json.as_bytes(), &ts_value_position, &json_ts);

        let dest_json = String::from_utf8(result).unwrap();

        println!("{}", dest_json_etalon);
        println!("{}", dest_json);

        assert_eq!(dest_json_etalon, dest_json);
    }

    #[test]
    fn test_replace_some_string_to_timestamp() {
        let json_ts = JsonTimeStamp::now();

        let src_json = r#"{"Field1":"Value1","TimeStamp":"ReplaceHere"}"#;

        let dest_json_etalon = format!(
            "{}\"Field1\":\"Value1\",\"TimeStamp\":\"{}\"{}",
            '{',
            json_ts.as_str(),
            '}'
        );

        let replace_string = "\"ReplaceHere\"";

        let index = src_json.find(replace_string).unwrap();

        let ts_value_position = TimeStampValuePosition {
            start: index,
            end: index + replace_string.len(),
        };

        let result =
            super::replace_timestamp_value(src_json.as_bytes(), &ts_value_position, &json_ts);

        let dest_json = String::from_utf8(result).unwrap();

        println!("{}", dest_json_etalon);
        println!("{}", dest_json);

        assert_eq!(dest_json_etalon, dest_json);
    }
}
