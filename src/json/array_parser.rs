pub use consts::*;

use super::consts;

pub struct JsonObjectsIterator<'t> {
    data: &'t [u8],
    start_index: usize,
    index: usize,
}

impl<'t> JsonObjectsIterator<'t> {
    pub fn new(data: &'t [u8]) -> Self {
        Self {
            data,
            start_index: 0,
            index: 0,
        }
    }
}

impl<'t> Iterator for JsonObjectsIterator<'t> {
    type Item = &'t [u8];

    fn next(&mut self) -> Option<Self::Item> {
        let mut escape_mode = false;
        let mut inside_string = false;
        let mut object_level: usize = 0;

        while self.index < self.data.len() {
            let b = self.data[self.index];

            if escape_mode {
                escape_mode = false;
                continue;
            }

            match b {
                ESC_SYMBOL => {
                    if inside_string {
                        escape_mode = true;
                    }
                }

                DOUBLE_QUOTE => {
                    inside_string = !inside_string;
                }

                OPEN_BRACKET => {
                    if !inside_string {
                        object_level = object_level + 1;
                        if object_level == 1 {
                            self.start_index = self.index
                        }
                    }
                }

                CLOSE_BRACKET => {
                    if !inside_string {
                        object_level = object_level - 1;
                        if object_level == 0 {
                            self.index += 1;
                            let slice = &self.data[self.start_index..self.index];
                            return Some(slice);
                        }
                    }
                }
                _ => {}
            }

            self.index += 1;
        }

        None
    }
}

pub trait ArrayToJsonObjectsSplitter<'t> {
    fn split_array_json_to_objects(self) -> JsonObjectsIterator<'t>;
}

impl<'t> ArrayToJsonObjectsSplitter<'t> for &'t [u8] {
    fn split_array_json_to_objects(self) -> JsonObjectsIterator<'t> {
        return JsonObjectsIterator::new(self);
    }
}

/*
pub fn split_to_objects<'t>(data: &'t [u8]) -> Result<Vec<DbEntity<'t>>, FailOperationResult> {
    let mut result: Vec<DbEntity<'t>> = Vec::new();
    for slice in data.split_json_to_objects() {
        let db_entity = DbEntity::<'t>::parse(slice)?;
        result.push(db_entity)
    }

    Ok(result)
}
*/

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    pub fn test_basic_json_array_split() {
        let json = r###"[{"id":1}, {"id":2}, {"id":3}]"###;

        let mut i = 0;
        for sub_json in json.as_bytes().split_array_json_to_objects() {
            i += 1;
            println!("{}", i);
            println!("{}", std::str::from_utf8(sub_json).unwrap());

            assert_eq!(
                format!("{{\"id\":{}}}", i),
                std::str::from_utf8(sub_json).unwrap()
            );
        }
    }
}
