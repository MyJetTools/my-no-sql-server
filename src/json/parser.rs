use std::collections::HashMap;

use super::{consts, JsonParseError};
use crate::json::JsonFirstLine;

enum ExpectedToken {
    OpenBracket,
    OpenKey,
    CloseKey,
    DoubleColumn,
    OpenValue,
    CloseStringValue,
    CloseNumberOrBoolValue,
    CloseObject,
    CloseArray,
    Comma,
    EndOfFile,
}

fn is_space(c: u8) -> bool {
    c <= 32
}

fn is_start_of_bool(c: u8) -> bool {
    c == 't' as u8
        || c == 'f' as u8
        || c == 'T' as u8
        || c == 'F' as u8
        || c == 'n' as u8
        || c == 'N' as u8
}

fn is_start_of_digit(c: u8) -> bool {
    if c == '-' as u8 {
        return true;
    }

    if c >= '0' as u8 && c <= '9' as u8 {
        return true;
    }

    return false;
}

pub fn parse_first_line<'s>(
    in_data: &'s [u8],
) -> Result<HashMap<&'s str, JsonFirstLine<'s>>, JsonParseError> {
    let mut result: HashMap<&'s str, JsonFirstLine<'s>> = HashMap::new();

    let mut expected_token = ExpectedToken::OpenBracket;

    let mut sub_object_level: usize = 0;
    let mut sub_object_string = false;
    let mut key_start_index: usize = 0;
    let mut key_end_index: usize = 0;
    let mut value_start_index: usize = 0;
    let mut skip_items = 0;

    for (index, b) in in_data.iter().enumerate() {
        let b = *b;
        if skip_items > 0 {
            skip_items = skip_items - 1;
            continue;
        }

        match expected_token {
            ExpectedToken::EndOfFile => {
                break;
            }

            ExpectedToken::OpenBracket => {
                if is_space(b) {
                    continue;
                }

                if b != consts::OPEN_BRACKET {
                    return Err(JsonParseError::new(
                        in_data,
                        index,
                        format!("Json parser expects '{}'", consts::OPEN_BRACKET),
                    ));
                }

                expected_token = ExpectedToken::OpenKey;
            }

            ExpectedToken::OpenKey => {
                if b == consts::CLOSE_BRACKET {
                    expected_token = ExpectedToken::EndOfFile;
                }

                if is_space(b) {
                    continue;
                }

                if b != consts::DOUBLE_QUOTE {
                    return Err(JsonParseError::new(
                        in_data,
                        index,
                        format!("Json parser expects '{}'", consts::DOUBLE_QUOTE),
                    ));
                }

                key_start_index = index;
                expected_token = ExpectedToken::CloseKey;
            }

            ExpectedToken::CloseKey => {
                match b {
                    consts::ESC_SYMBOL => {
                        skip_items = skip_items + 1;
                    }

                    consts::DOUBLE_QUOTE => {
                        key_end_index = index + 1;
                        expected_token = ExpectedToken::DoubleColumn;
                    }
                    _ => {}
                };
            }

            ExpectedToken::DoubleColumn => {
                if is_space(b) {
                    continue;
                }

                if b != consts::DOUBLE_COLUMN {
                    return Err(JsonParseError::new(
                        in_data,
                        index,
                        format!("Json parser expects '{}'", consts::DOUBLE_COLUMN),
                    ));
                }

                expected_token = ExpectedToken::OpenValue;
            }

            ExpectedToken::OpenValue => {
                if is_space(b) {
                    continue;
                }

                value_start_index = index;

                match b {
                    consts::OPEN_ARRAY => {
                        expected_token = ExpectedToken::CloseArray;
                    }

                    consts::DOUBLE_QUOTE => {
                        expected_token = ExpectedToken::CloseStringValue;
                    }

                    consts::OPEN_BRACKET => {
                        sub_object_level = 0;
                        sub_object_string = false;
                        expected_token = ExpectedToken::CloseObject;
                    }
                    _ => {
                        if is_start_of_digit(b) || is_start_of_bool(b) {
                            expected_token = ExpectedToken::CloseNumberOrBoolValue;
                        } else {
                            return Err(JsonParseError::new(
                                in_data,
                                index,
                                "Json parser expects Close number or boolean value".to_string(),
                            ));
                        }
                    }
                }
            }

            ExpectedToken::CloseStringValue => match b {
                consts::ESC_SYMBOL => {
                    skip_items = skip_items + 1;
                }

                consts::DOUBLE_QUOTE => {
                    let itm = JsonFirstLine {
                        name_start: key_start_index,
                        name_end: key_end_index,
                        value_start: value_start_index,
                        value_end: index + 1,
                        data: in_data,
                    };

                    let key = itm.get_name()?;
                    result.insert(key, itm);

                    expected_token = ExpectedToken::Comma;
                }
                _ => {}
            },

            ExpectedToken::CloseNumberOrBoolValue => {
                if b == consts::COMMA || b == consts::CLOSE_BRACKET || is_space(b) {
                    let itm = JsonFirstLine {
                        name_start: key_start_index,
                        name_end: key_end_index,
                        value_start: value_start_index,
                        value_end: index,
                        data: in_data,
                    };

                    let key = itm.get_name()?;
                    result.insert(key, itm);

                    if b == consts::CLOSE_BRACKET {
                        expected_token = ExpectedToken::EndOfFile;
                    } else {
                        expected_token = match b {
                            consts::COMMA => ExpectedToken::OpenKey,
                            _ => ExpectedToken::Comma,
                        }
                    }
                }
            }

            ExpectedToken::Comma => {
                if is_space(b) {
                    continue;
                }

                if b == consts::CLOSE_BRACKET {
                    expected_token = ExpectedToken::EndOfFile;
                    continue;
                }

                if b != consts::COMMA {
                    return Err(JsonParseError::new(
                        in_data,
                        index,
                        format!("Json parser expects {}", consts::COMMA),
                    ));
                }

                expected_token = ExpectedToken::OpenKey;
            }

            ExpectedToken::CloseObject => {
                if sub_object_string {
                    match b {
                        consts::ESC_SYMBOL => {
                            skip_items = skip_items + 1;
                        }
                        consts::DOUBLE_QUOTE => {
                            sub_object_string = false;
                        }
                        _ => {}
                    }
                } else {
                    match b {
                        consts::DOUBLE_QUOTE => {
                            sub_object_string = true;
                        }
                        consts::OPEN_BRACKET => {
                            sub_object_level = sub_object_level + 1;
                        }
                        consts::CLOSE_BRACKET => {
                            if sub_object_level == 0 {
                                let itm = JsonFirstLine {
                                    name_start: key_start_index,
                                    name_end: key_end_index,
                                    value_start: value_start_index,
                                    value_end: index + 1,
                                    data: in_data,
                                };

                                let key = itm.get_name()?;

                                result.insert(key, itm);

                                expected_token = ExpectedToken::Comma;
                            } else {
                                sub_object_level = sub_object_level - 1;
                            }
                        }
                        _ => {}
                    }
                }
            }

            ExpectedToken::CloseArray => {
                if sub_object_string {
                    match b {
                        consts::ESC_SYMBOL => {
                            skip_items = skip_items + 1;
                        }
                        consts::DOUBLE_QUOTE => {
                            sub_object_string = false;
                        }
                        _ => {}
                    }
                } else {
                    match b {
                        consts::DOUBLE_QUOTE => {
                            sub_object_string = true;
                        }
                        consts::OPEN_ARRAY => {
                            sub_object_level = sub_object_level + 1;
                        }

                        consts::CLOSE_ARRAY => {
                            if sub_object_level == 0 {
                                let itm = JsonFirstLine {
                                    name_start: key_start_index,
                                    name_end: key_end_index,
                                    value_start: value_start_index,
                                    value_end: index + 1,
                                    data: in_data,
                                };

                                let key = itm.get_name()?;
                                result.insert(key, itm);

                                expected_token = ExpectedToken::Comma;
                            } else {
                                sub_object_level = sub_object_level - 1;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    match expected_token {
        ExpectedToken::EndOfFile => Ok(result),
        _ => Err(JsonParseError::new(in_data, 0, "Invalid Json".to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_simple_parse() {
        let src_data = "{\"name1\":\"123\", \"name2\":true}".to_string();

        let src_data = src_data.into_bytes();

        let mut res = parse_first_line(src_data.as_ref()).ok().unwrap();

        println!("Console: {}", res.len());

        for itm in res.drain() {
            println!("{} / {}", itm.0, itm.1.get_value().ok().unwrap());
        }
    }

    #[test]
    pub fn test_second_case() {
        let src_data = r###"{"Value":"Test2","PartitionKey":"Pk1","RowKey":"Rk1","TimeStamp":"2021-06-22T20:34:05.4741090Z","Expires":null}"###;

        let src_data = src_data.as_bytes();

        let mut res = parse_first_line(src_data.as_ref()).ok().unwrap();

        println!("Console: {}", res.len());

        for itm in res.drain() {
            println!(
                "{} = {}",
                std::str::from_utf8(&itm.1.data[itm.1.name_start..itm.1.name_end]).unwrap(),
                std::str::from_utf8(&itm.1.data[itm.1.value_start..itm.1.value_end]).unwrap()
            );

            println!("{} = {}", itm.0, itm.1.get_value().ok().unwrap());
        }
    }
}
