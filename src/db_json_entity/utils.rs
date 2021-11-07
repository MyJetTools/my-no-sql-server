use rust_extensions::date_time::{utils::parse_iso_string, DateTimeAsMicroseconds};

pub struct JsonTimeStamp {
    pub date_time: DateTimeAsMicroseconds,
    str_value: String,
    index: usize,
}

impl JsonTimeStamp {
    pub fn now() -> Self {
        let date_time = DateTimeAsMicroseconds::now();
        let str_value = date_time.to_rfc3339();
        let index = find_end_of_the_string(&str_value);

        Self {
            date_time,
            str_value,
            index,
        }
    }

    pub fn parse_or_now(src: &str) -> Self {
        let dt = parse_iso_string(src.as_bytes());

        let result = if let Some(dt) = dt {
            let result = DateTimeAsMicroseconds::new(dt);
            let str_value = src.to_string();
            (result, str_value)
        } else {
            let result = DateTimeAsMicroseconds::now();
            let str_value = result.to_rfc3339();
            (result, str_value)
        };

        let index = find_end_of_the_string(&result.1);

        Self {
            date_time: result.0,
            str_value: result.1,
            index,
        }
    }

    pub fn as_str(&self) -> &str {
        return std::str::from_utf8(self.as_slice()).unwrap();
    }

    pub fn as_slice(&self) -> &[u8] {
        return &self.str_value.as_bytes()[..self.index];
    }
}

const ZERO: u8 = '0' as u8;
const NINE: u8 = '0' as u8;
fn find_end_of_the_string(src: &str) -> usize {
    let bytes = src.as_bytes();

    for i in 24..bytes.len() {
        let b = bytes[i];
        if b < ZERO || b > NINE {
            return i;
        }
    }

    return src.len();
}

#[cfg(test)]
mod tests {
    use super::JsonTimeStamp;

    #[test]
    fn test_parse_dt() {
        let ts = JsonTimeStamp::parse_or_now("2020-01-01T00:00:00.123");

        assert_eq!("2020-01-01T00:00:00.123", ts.as_str());
    }

    #[test]
    fn test_parse_dt_2() {
        let ts = JsonTimeStamp::parse_or_now("2020-01-01T00:00:00.1234");

        assert_eq!("2020-01-01T00:00:00.1234", ts.as_str());
    }

    #[test]
    fn test_parse_dt_3() {
        let ts = JsonTimeStamp::parse_or_now("2020-01-01T00:00:00");

        assert_eq!("2020-01-01T00:00:00", ts.as_str());
    }
}
