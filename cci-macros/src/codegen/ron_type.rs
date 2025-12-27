use proc_macro2::TokenStream;
use quote::{quote, format_ident};
use crate::preset::PresetFieldOpts;

pub fn generate_ron_type(
    preset_ident: &syn::Ident,
    _preset_id: &str,
    fields: &[PresetFieldOpts],
) -> TokenStream {
    // Generate RON config struct name (e.g., RustPreset -> RustConfig)
    let config_name = format_ident!("{}Config", preset_ident.to_string().strip_suffix("Preset").unwrap_or(&preset_ident.to_string()));

    // Generate fields for the RON struct
    let ron_fields = fields.iter().map(|field| {
        let field_ident = field.ident.as_ref().unwrap();
        let field_ty = &field.ty;

        // Determine the RON field name
        let ron_field_name = if let Some(ref ron_field) = field.ron_field {
            format_ident!("{}", ron_field)
        } else if let Some(ref id) = field.id {
            // Strip "enable_" prefix if present for RON field
            let cleaned = id.strip_prefix("enable_").unwrap_or(id);
            format_ident!("{}", cleaned)
        } else {
            // Use the rust field name, strip enable_ prefix
            let field_str = field_ident.to_string();
            let cleaned = field_str.strip_prefix("enable_").unwrap_or(&field_str);
            format_ident!("{}", cleaned)
        };

        // Add #[serde(default)] for non-String types
        let serde_default = match field_ty {
            syn::Type::Path(type_path) => {
                let type_str = quote!(#type_path).to_string().replace(" ", "");
                // Don't add default for basic String types, but do for Vec and other types
                if type_str == "String" {
                    quote! {}
                } else {
                    quote! { #[serde(default)] }
                }
            }
            _ => quote! { #[serde(default)] },
        };

        quote! {
            #serde_default
            pub #ron_field_name: #field_ty
        }
    });

    quote! {
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
        pub struct #config_name {
            #(#ron_fields),*
        }
    }
}
