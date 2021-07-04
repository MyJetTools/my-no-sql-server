use crate::utils::StringBuilder;

#[derive(Debug)]
pub struct JsonParseError {
    pub pos: usize,
    pub msg: String,
    pub json_before: Option<Vec<u8>>,
    pub json_after: Option<Vec<u8>>,
    pub invalid_symbol: u8,
}

impl JsonParseError {
    pub fn new(in_data: &[u8], pos: usize, msg: String) -> Self {
        let mut i = pos as i32 - 20;

        let mut json_before = None;

        if i < 0 {
            i = 0
        }

        let i = i as usize;

        if i < pos {
            let i = i as usize;
            let slice_before = &in_data[i..pos];
            json_before = Some(slice_before.to_vec());
        }

        let mut i = pos + 20;

        if i > in_data.len() {
            i = in_data.len();
        }

        let mut json_after = None;

        if pos < i {
            let s = &in_data[pos + 1..i];
            if s.len() > 0 {
                json_after = Some(s.to_vec());
            }
        }

        JsonParseError {
            pos,
            msg,
            json_before,
            json_after,
            invalid_symbol: in_data[pos],
        }
    }

    pub fn to_string(&self) -> String {
        let mut sb = StringBuilder::new();

        sb.append_line(self.msg.as_str());
        sb.append_line(format!("Pos: {}", self.pos).as_str());

        if let Some(before) = &self.json_before {
            sb.append_bytes(before);
            sb.append(" -->");
        }

        sb.append_u8(self.invalid_symbol);

        if let Some(after) = &self.json_after {
            sb.append("<-- ");
            sb.append_bytes(after.as_slice());
        }

        return sb.to_string_utf8().unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_to_string() {
        let str = "1234567";

        let err = JsonParseError::new(str.as_bytes(), 3, "Test Error".to_string());

        println!("{}", err.to_string());
    }

    #[test]
    pub fn test_to_string_beginning() {
        let str = "1234567";

        let err = JsonParseError::new(str.as_bytes(), 0, "Test Error".to_string());

        println!("{}", err.to_string());
    }

    #[test]
    pub fn test_to_string_end() {
        let str = "1234567";

        let err = JsonParseError::new(str.as_bytes(), 6, "Test Error".to_string());

        println!("{:?}", err);
        println!("{}", err.to_string());
    }

    #[test]
    pub fn test_to_string_end1() {
        let str = "1234567";

        let err = JsonParseError::new(str.as_bytes(), 5, "Test Error".to_string());

        println!("{:?}", err);
        println!("{}", err.to_string());
    }
}
