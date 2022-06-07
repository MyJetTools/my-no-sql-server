use rust_extensions::date_time::DateTimeAsMicroseconds;

pub enum JsonValue<'s> {
    Null,
    String(&'s str),
    Number(&'s str),
    Boolean(bool),
    Array(&'s [u8]),
    Object(&'s [u8]),
}

impl<'s> JsonValue<'s> {
    pub fn as_date_time(&self) -> Option<DateTimeAsMicroseconds> {
        crate::json::date_time::parse(self.as_str()?.as_bytes())
    }

    pub fn as_str(&self) -> Option<&'s str> {
        match self {
            JsonValue::Null => None,
            JsonValue::String(src) => Some(src[1..src.len() - 1].as_ref()),
            JsonValue::Number(src) => Some(src),
            JsonValue::Boolean(src) => match src {
                true => Some("true"),
                false => Some("false"),
            },
            JsonValue::Array(_) => {
                panic!("Json array can no be converted to string. Does not make sence")
            }
            JsonValue::Object(_) => {
                panic!("Json object can no be converted to string. Does not make sence")
            }
        }
    }

    /*
    pub fn as_bytes(&self) -> Option<&'s [u8]> {
        match self {
            JsonValue::Null => None,
            JsonValue::String(src) => Some(src.as_bytes()),
            JsonValue::Number(src) => Some(src.as_bytes()),
            JsonValue::Boolean(src) => match src {
                true => Some("true".as_bytes()),
                false => Some("false".as_bytes()),
            },
            JsonValue::Array(src) => Some(src),
            JsonValue::Object(src) => Some(src),
        }
    }
     */
}
