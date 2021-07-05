pub enum ExpectedToken {
    OpenBracket,
    OpenKey,
    CloseKey,
    DoubleColumn,
    OpenValue,
    CloseStringValue,
    CloseNumberOrBoolOrNullValue,
    CloseObject,
    CloseArray,
    Comma,
    EndOfFile,
}

pub fn is_space(c: u8) -> bool {
    c <= 32
}

pub fn is_start_of_bool_or_null(c: u8) -> bool {
    c == 't' as u8
        || c == 'f' as u8
        || c == 'T' as u8
        || c == 'F' as u8
        || c == 'n' as u8
        || c == 'N' as u8
}

pub fn is_start_of_digit(c: u8) -> bool {
    if c == '-' as u8 {
        return true;
    }

    if c >= '0' as u8 && c <= '9' as u8 {
        return true;
    }

    return false;
}
