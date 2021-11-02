use crate::json::JsonParseError;

#[derive(Debug)]
pub enum DbEntityParseFail {
    FieldPartitionKeyIsRequired,
    FieldRowKeyIsRequired,
    JsonParseError(JsonParseError),
}

impl From<JsonParseError> for DbEntityParseFail {
    fn from(src: JsonParseError) -> Self {
        Self::JsonParseError(src)
    }
}
