use super::consts;
use super::utils::*;
use super::JsonFirstLine;
use crate::json::JsonParseError;
pub struct JsonFirstLineParser<'s> {
    in_data: &'s [u8],
    index: usize,
    key_start_index: usize,
    key_end_index: usize,
    value_start_index: usize,
    expected_token: ExpectedToken,
}

impl<'s> JsonFirstLineParser<'s> {
    pub fn new(in_data: &'s [u8]) -> Self {
        Self {
            in_data,
            index: 0,
            key_start_index: 0,
            key_end_index: 0,
            value_start_index: 0,
            expected_token: ExpectedToken::OpenBracket,
        }
    }
}

impl<'s> Iterator for JsonFirstLineParser<'s> {
    type Item = Result<JsonFirstLine<'s>, JsonParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut sub_object_level: usize = 0;
        let mut sub_object_string = false;
        let mut skip_items = 0;

        while self.index < self.in_data.len() {
            let b = self.in_data[self.index];

            if skip_items > 0 {
                skip_items = skip_items - 1;
                self.index += 1;
                continue;
            }

            match self.expected_token {
                ExpectedToken::EndOfFile => {
                    break;
                }

                ExpectedToken::OpenBracket => {
                    if is_space(b) {
                        self.index += 1;
                        continue;
                    }

                    if b != consts::OPEN_BRACKET {
                        let err = Err(JsonParseError::new(
                            self.in_data,
                            self.index,
                            format!("Json parser expects '{}'", consts::OPEN_BRACKET),
                        ));

                        return Some(err);
                    }

                    self.expected_token = ExpectedToken::OpenKey;
                }

                ExpectedToken::OpenKey => {
                    if b == consts::CLOSE_BRACKET {
                        self.expected_token = ExpectedToken::EndOfFile;
                    }

                    if is_space(b) {
                        self.index += 1;
                        continue;
                    }

                    if b != consts::DOUBLE_QUOTE {
                        let err = Err(JsonParseError::new(
                            self.in_data,
                            self.index,
                            format!("Json parser expects '{}'", consts::DOUBLE_QUOTE),
                        ));

                        return Some(err);
                    }

                    self.key_start_index = self.index;
                    self.expected_token = ExpectedToken::CloseKey;
                }

                ExpectedToken::CloseKey => {
                    match b {
                        consts::ESC_SYMBOL => {
                            skip_items = skip_items + 1;
                        }

                        consts::DOUBLE_QUOTE => {
                            self.key_end_index = self.index + 1;
                            self.expected_token = ExpectedToken::DoubleColumn;
                        }
                        _ => {}
                    };
                }

                ExpectedToken::DoubleColumn => {
                    if is_space(b) {
                        self.index += 1;
                        continue;
                    }

                    if b != consts::DOUBLE_COLUMN {
                        let err = Err(JsonParseError::new(
                            self.in_data,
                            self.index,
                            format!("Json parser expects '{}'", consts::DOUBLE_COLUMN),
                        ));

                        return Some(err);
                    }

                    self.expected_token = ExpectedToken::OpenValue;
                }

                ExpectedToken::OpenValue => {
                    if is_space(b) {
                        self.index += 1;
                        continue;
                    }

                    self.value_start_index = self.index;

                    match b {
                        consts::OPEN_ARRAY => {
                            self.expected_token = ExpectedToken::CloseArray;
                        }

                        consts::DOUBLE_QUOTE => {
                            self.expected_token = ExpectedToken::CloseStringValue;
                        }

                        consts::OPEN_BRACKET => {
                            sub_object_level = 0;
                            sub_object_string = false;
                            self.expected_token = ExpectedToken::CloseObject;
                        }
                        _ => {
                            if is_start_of_digit(b) || is_start_of_bool_or_null(b) {
                                self.expected_token = ExpectedToken::CloseNumberOrBoolOrNullValue;
                            } else {
                                let err = Err(JsonParseError::new(
                                    self.in_data,
                                    self.index,
                                    "Json parser expects Close number or boolean value".to_string(),
                                ));

                                return Some(err);
                            }
                        }
                    }
                }

                ExpectedToken::CloseStringValue => match b {
                    consts::ESC_SYMBOL => {
                        skip_items = skip_items + 1;
                    }

                    consts::DOUBLE_QUOTE => {
                        self.index += 1;

                        let itm = JsonFirstLine {
                            name_start: self.key_start_index,
                            name_end: self.key_end_index,
                            value_start: self.value_start_index,
                            value_end: self.index,
                            data: self.in_data,
                        };

                        self.expected_token = ExpectedToken::Comma;

                        return Some(Ok(itm));
                    }
                    _ => {}
                },

                ExpectedToken::CloseNumberOrBoolOrNullValue => {
                    if b == consts::COMMA || b == consts::CLOSE_BRACKET || is_space(b) {
                        let itm = JsonFirstLine {
                            name_start: self.key_start_index,
                            name_end: self.key_end_index,
                            value_start: self.value_start_index,
                            value_end: self.index,
                            data: self.in_data,
                        };

                        self.index += 1;

                        if b == consts::CLOSE_BRACKET {
                            self.expected_token = ExpectedToken::EndOfFile;
                        } else {
                            self.expected_token = match b {
                                consts::COMMA => ExpectedToken::OpenKey,
                                _ => ExpectedToken::Comma,
                            }
                        }

                        return Some(Ok(itm));
                    }
                }

                ExpectedToken::Comma => {
                    if is_space(b) {
                        self.index += 1;
                        continue;
                    }

                    if b == consts::CLOSE_BRACKET {
                        self.expected_token = ExpectedToken::EndOfFile;
                        self.index += 1;
                        continue;
                    }

                    if b != consts::COMMA {
                        let err = Err(JsonParseError::new(
                            self.in_data,
                            self.index,
                            format!("Json parser expects {}", consts::COMMA),
                        ));

                        return Some(err);
                    }

                    self.expected_token = ExpectedToken::OpenKey;
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
                                self.index += 1;
                                if sub_object_level == 0 {
                                    let itm = JsonFirstLine {
                                        name_start: self.key_start_index,
                                        name_end: self.key_end_index,
                                        value_start: self.value_start_index,
                                        value_end: self.index,
                                        data: self.in_data,
                                    };

                                    self.expected_token = ExpectedToken::Comma;

                                    return Some(Ok(itm));
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
                                    self.index += 1;
                                    let itm = JsonFirstLine {
                                        name_start: self.key_start_index,
                                        name_end: self.key_end_index,
                                        value_start: self.value_start_index,
                                        value_end: self.index,
                                        data: self.in_data,
                                    };

                                    self.expected_token = ExpectedToken::Comma;

                                    return Some(Ok(itm));
                                } else {
                                    sub_object_level = sub_object_level - 1;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            self.index += 1;
        }

        match self.expected_token {
            ExpectedToken::EndOfFile => None,
            _ => Some(Err(JsonParseError::new(
                self.in_data,
                self.index,
                "Invalid Json".to_string(),
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_simple_parse() {
        let src_data = "{\"name1\":\"123\", \"name2\":true,       \"name3\":null}";

        let mut parser = JsonFirstLineParser::new(src_data.as_bytes());

        let item = parser.next().unwrap().unwrap();

        assert_eq!("\"name1\"", item.get_raw_name().unwrap());
        assert_eq!("\"123\"", item.get_raw_value().unwrap());

        let item = parser.next().unwrap().unwrap();

        assert_eq!("\"name2\"", item.get_raw_name().unwrap());
        assert_eq!("true", item.get_raw_value().unwrap());

        let item = parser.next().unwrap().unwrap();

        assert_eq!("\"name3\"", item.get_raw_name().unwrap());
        assert_eq!("null", item.get_raw_value().unwrap());

        let item = parser.next();

        assert_eq!(true, item.is_none());
    }
}
