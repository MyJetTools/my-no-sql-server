pub struct AttrParamsParser {
    line: Vec<u8>,

    key_start: usize,
    key_end: usize,
    value_start: usize,
    value_end: usize,
}

impl AttrParamsParser {
    pub fn new(line: Vec<u8>) -> Self {
        Self {
            line,
            key_start: 0,
            key_end: 0,
            value_start: 0,
            value_end: 0,
        }
    }

    pub fn find_key_start(&mut self) -> Option<()> {
        for i in self.key_start..self.line.len() {
            if self.line[i] == '(' as u8 {
                continue;
            }

            if self.line[i] == '*' as u8 {
                continue;
            }

            if self.line[i] == ';' as u8 {
                continue;
            }

            if self.line[i] == 32 {
                continue;
            }

            self.key_start = i;
            return Some(());
        }

        None
    }

    fn fine_key_end(&mut self) -> Option<()> {
        for i in self.key_start..self.line.len() {
            let b = self.line[i];

            if b == '=' as u8 || b == 32 {
                self.key_end = i;
                return Some(());
            }
        }

        None
    }

    fn fine_value_start(&mut self) -> Option<()> {
        for i in self.key_end..self.line.len() {
            let b = self.line[i];

            if b == '=' as u8 {
                continue;
            }

            if b == '"' as u8 {
                self.value_start = i + 1;
                return Some(());
            }
        }

        None
    }

    fn fine_value_end(&mut self) -> Option<()> {
        for i in self.value_start..self.line.len() {
            let b = self.line[i];

            if b == '"' as u8 {
                self.value_end = i;
                return Some(());
            }
        }

        None
    }
}

impl Iterator for AttrParamsParser {
    type Item = (String, String);

    fn next(&mut self) -> Option<Self::Item> {
        self.key_start = self.value_end + 1;

        self.find_key_start()?;

        self.fine_key_end()?;
        self.fine_value_start()?;
        self.fine_value_end()?;

        let key = &self.line[self.key_start..self.key_end];

        let key = String::from_utf8(key.to_vec()).unwrap();

        let value = &self.line[self.value_start..self.value_end];

        let value = String::from_utf8(value.to_vec()).unwrap();

        Some((key, value))
    }
}

#[cfg(test)]
mod test {
    use crate::parser::AttrParamsParser;

    #[test]
    fn test_basic_case() {
        let line = "(name=\"AAA\"; description=\"CCC\")";

        let parser = AttrParamsParser::new(line.as_bytes().to_vec());

        let result: Vec<(String, String)> = parser.collect();

        assert_eq!(2, result.len());

        assert_eq!("name", result[0].0);
        assert_eq!("AAA", result[0].1);
        assert_eq!("description", result[1].0);
        assert_eq!("CCC", result[1].1);
    }

    #[test]
    fn test_description_case() {
        let line = "(description=\"CCC\")";

        let parser = AttrParamsParser::new(line.as_bytes().to_vec());

        let result: Vec<(String, String)> = parser.collect();

        assert_eq!(1, result.len());

        assert_eq!("description", result[0].0);
        assert_eq!("CCC", result[0].1);
    }
}
