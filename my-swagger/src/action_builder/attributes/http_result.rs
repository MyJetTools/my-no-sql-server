use super::ResultType;

pub struct HttpResult {
    pub status_code: u16,
    pub description: String,
    pub result_type: Option<ResultType>,
}

impl HttpResult {
    pub fn new(src: Option<String>) -> Vec<HttpResult> {
        let mut result = vec![];

        if src.is_none() {
            return result;
        }

        for fields in JsonObjectsSimpleScanner::new(src.as_ref().unwrap().as_bytes()) {
            let mut status_code: Option<u16> = None;
            let mut description: Option<String> = None;
            let mut model: Option<String> = None;
            let mut model_as_array: Option<String> = None;

            for field in fields.split(',') {
                let kv: Vec<&str> = field.split(':').collect();

                if kv.len() < 2 {
                    continue;
                }

                match kv[0].trim() {
                    "status_code" => {
                        status_code = Some(kv[1].trim().parse::<u16>().unwrap());
                    }
                    "description" => {
                        description = Some(remove_quotes(kv[1].trim()));
                    }
                    "model" => {
                        model = Some(remove_quotes(kv[1].trim()));
                    }
                    "model_as_array" => {
                        model_as_array = Some(remove_quotes(kv[1].trim()));
                    }
                    _ => {
                        continue;
                    }
                }
            }

            if status_code.is_none() {
                panic!("status_code is not found");
            }

            if description.is_none() {
                panic!("description is not found");
            }

            result.push(HttpResult {
                status_code: status_code.unwrap(),
                description: description.unwrap(),
                result_type: ResultType::new(model, model_as_array),
            });
        }

        result
    }
}

fn remove_quotes(src: &str) -> String {
    src[1..src.len() - 1].to_string()
}

pub struct JsonObjectsSimpleScanner<'s> {
    content: &'s [u8],
    pos: usize,
}

impl<'s> JsonObjectsSimpleScanner<'s> {
    pub fn new(content: &'s [u8]) -> Self {
        Self { content, pos: 0 }
    }
}

impl<'s> Iterator for JsonObjectsSimpleScanner<'s> {
    type Item = &'s str;

    fn next(&mut self) -> Option<Self::Item> {
        let start_pos = find(self.content, '{' as u8, self.pos)?;
        let end_pos = find(self.content, '}' as u8, start_pos + 1);

        let end_pos = end_pos?;

        self.pos = end_pos + 1;

        Some(std::str::from_utf8(&self.content[start_pos + 1..end_pos]).unwrap())
    }
}

pub fn find(src: &[u8], symbol: u8, start_pos: usize) -> Option<usize> {
    for i in start_pos..src.len() {
        if src[i] == symbol {
            return Some(i);
        }
    }

    None
}
