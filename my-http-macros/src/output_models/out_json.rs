use crate::reflection::StructProperty;
pub struct OutputJson {
    pub fields: Vec<JsonField>,
}

impl OutputJson {
    pub fn new(properties: Vec<StructProperty>) -> Self {
        let mut fields = Vec::new();

        for property in properties {
            fields.push(JsonField::new(property))
        }

        Self { fields }
    }
}

pub struct JsonField {
    pub property: StructProperty,
}

impl JsonField {
    pub fn new(property: StructProperty) -> Self {
        Self { property }
    }

    pub fn name(&self) -> &str {
        if let Some(attr) = self.property.attrs.get("serde") {
            if let Some(value) = attr.get_value("rename") {
                return value;
            } else {
                return self.property.name.as_str();
            }
        }

        self.property.name.as_str()
    }
}
