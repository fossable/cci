/// Classification of a preset field's type, used to pick the matching
/// `OptionValue` representation in generated code.
pub enum FieldKind {
    Bool,
    String,
    /// `Option<T>` where `T` is a strum-derived enum; carries the inner type.
    OptionEnum(Box<syn::Type>),
    /// A bare strum-derived enum.
    Enum,
    /// Anything else (e.g. `Vec<T>`); not exposed as an editor option.
    Other,
}

impl FieldKind {
    pub fn of(ty: &syn::Type) -> Self {
        let syn::Type::Path(type_path) = ty else {
            return FieldKind::Other;
        };
        let type_str = quote::quote!(#type_path).to_string().replace(' ', "");
        if type_str == "bool" {
            FieldKind::Bool
        } else if type_str == "String" {
            FieldKind::String
        } else if type_str.starts_with("Vec<") {
            FieldKind::Other
        } else if let Some(inner) = type_str
            .strip_prefix("Option<")
            .and_then(|s| s.strip_suffix('>'))
        {
            match syn::parse_str::<syn::Type>(inner) {
                Ok(inner_ty) => FieldKind::OptionEnum(Box::new(inner_ty)),
                Err(_) => FieldKind::Other,
            }
        } else {
            FieldKind::Enum
        }
    }
}
