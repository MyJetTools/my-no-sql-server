use crate::reflection::{MyAttribute, StructProperty};

pub enum InputFieldSource {
    Query,
    Path,
    Header,
    Body,
    Form,
}

impl InputFieldSource {
    pub fn parse(src: &str) -> Option<Self> {
        match src {
            "http_query" => Some(Self::Query),
            "http_header" => Some(Self::Header),
            "http_path" => Some(Self::Path),
            "http_form" => Some(Self::Form),
            "http_body" => Some(Self::Body),
            _ => None,
        }
    }

    pub fn is_body(&self) -> bool {
        match self {
            InputFieldSource::Body => true,
            _ => false,
        }
    }
}

pub struct InputField {
    pub property: StructProperty,
    pub src: InputFieldSource,
    pub my_attr: MyAttribute,
}

fn get_attr(property: &StructProperty) -> Option<(MyAttribute, InputFieldSource)> {
    for attr in property.attrs.values() {
        let src = InputFieldSource::parse(attr.name.as_str());

        if let Some(src) = src {
            return Some((attr.clone(), src));
        }
    }
    None
}

impl InputField {
    pub fn new(property: StructProperty) -> Option<Self> {
        let (my_attr, src) = get_attr(&property)?;

        return Self {
            property,
            src,
            my_attr,
        }
        .into();
    }

    pub fn name(&self) -> &str {
        if let Some(value) = self.my_attr.get_value("name") {
            value
        } else {
            self.property.name.as_str()
        }
    }

    pub fn required(&self) -> bool {
        !self.property.ty.is_option()
    }

    pub fn default(&self) -> Option<&str> {
        self.my_attr.get_value("default")
    }

    pub fn is_body(&self) -> bool {
        if let InputFieldSource::Body = self.src {
            return true;
        }

        return false;
    }

    pub fn description(&self) -> &str {
        if let Some(value) = self.my_attr.get_value("description") {
            return value;
        }

        panic!(
            "Description field is missing of the field {}",
            self.property.name
        );
    }

    pub fn struct_field_name(&self) -> &str {
        self.property.name.as_str()
    }
}

pub struct InputFields {
    pub fields: Vec<InputField>,
}

impl InputFields {
    pub fn new(src: Vec<StructProperty>) -> Self {
        let mut fields = Vec::new();

        for prop in src {
            if let Some(field) = InputField::new(prop) {
                fields.push(field);
            }
        }

        Self { fields }
    }

    pub fn has_query(&self) -> bool {
        for field in &self.fields {
            if let InputFieldSource::Query = &field.src {
                return true;
            }
        }
        return false;
    }

    pub fn has_form_data(&self) -> bool {
        for field in &self.fields {
            if let InputFieldSource::Form = &field.src {
                return true;
            }
        }
        return false;
    }

    pub fn get_body_field<'s>(&'s self) -> Option<&'s InputField> {
        for field in &self.fields {
            if field.src.is_body() {
                return Some(field);
            }
        }

        None
    }
}
