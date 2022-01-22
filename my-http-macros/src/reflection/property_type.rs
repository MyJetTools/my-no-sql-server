pub struct PropertyType {
    pub raw: String,

    pub tp: syn::TypePath,
}

impl PropertyType {
    pub fn new(field: &syn::Field) -> Self {
        let tp = if let syn::Type::Path(tp) = &field.ty {
            tp.clone()
        } else {
            panic!("Type is not struct");
        };

        Self {
            raw: get_http_type(field),
            tp,
        }
    }

    pub fn get_generic(&self) -> PropertyType {
        for path in &self.tp.path.segments {
            if let syn::PathArguments::AngleBracketed(args) = &path.arguments {
                for arg in &args.args {
                    if let syn::GenericArgument::Type(ty) = &arg {
                        if let syn::Type::Path(tp) = ty {
                            for path in &tp.path.segments {
                                return PropertyType {
                                    raw: path.ident.to_string(),
                                    tp: tp.clone(),
                                };
                            }
                        }
                    }
                }
            }
        }

        panic!("Can not get generic from the type {}", self.raw);
    }

    pub fn is_string(&self) -> bool {
        self.raw == "String"
    }

    pub fn is_option(&self) -> bool {
        self.raw == "Option"
    }
}

fn get_http_type(field: &syn::Field) -> String {
    match &field.ty {
        syn::Type::Slice(_) => panic!("Slice type is not supported"),
        syn::Type::Array(_) => panic!("Array type is not supported"),
        syn::Type::Ptr(_) => panic!("Ptr type is not supported"),
        syn::Type::Reference(_) => panic!("Reference type is not supported"),
        syn::Type::BareFn(_) => panic!("BareFn type is not supported"),
        syn::Type::Never(_) => panic!("Never type is not supported"),
        syn::Type::Tuple(_) => panic!("Tuple type is not supported"),
        syn::Type::Path(type_path) => get_type_as_string(type_path),
        syn::Type::TraitObject(_) => panic!("TraitObject type is not supported"),
        syn::Type::ImplTrait(_) => panic!("ImplTrait type is not supported"),
        syn::Type::Paren(_) => panic!("Paren type is not supported"),
        syn::Type::Group(_) => panic!("Group type is not supported"),
        syn::Type::Infer(_) => panic!("Infer type is not supported"),
        syn::Type::Macro(_) => panic!("Macro type is not supported"),
        syn::Type::Verbatim(_) => panic!("Verbatim type is not supported"),
        syn::Type::__TestExhaustive(_) => panic!("__TestExhaustive type is not supported"),
    }
}

fn get_type_as_string(field: &syn::TypePath) -> String {
    let mut result = None;
    for segment in &field.path.segments {
        result = Some(segment);
    }

    let result = result.unwrap();

    return result.ident.to_string();
}
