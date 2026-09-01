use crate::codegen::FieldKind;
use crate::preset::PresetFieldOpts;
use proc_macro2::TokenStream;
use quote::quote;

pub fn generate_conversions(
    preset_ident: &syn::Ident,
    preset_id: &str,
    fields: &[PresetFieldOpts],
) -> TokenStream {
    let from_config_impl = generate_from_config(fields);
    let to_preset_config_impl = generate_to_preset_config(preset_id, fields);

    quote! {
        impl #preset_ident {
            #from_config_impl
            #to_preset_config_impl
        }
    }
}

fn generate_from_config(fields: &[PresetFieldOpts]) -> TokenStream {
    let field_assignments = fields.iter().map(|field| {
        let field_ident = field.ident.as_ref().unwrap();
        let option_id = field_ident.to_string();

        // Hidden fields never live in PresetConfig. Fields named `*_version`
        // take the language version when one is known; everything else keeps
        // its default.
        if field.hidden {
            if option_id.ends_with("_version") {
                return quote! {
                    #field_ident: if version.is_empty() {
                        d.#field_ident.clone()
                    } else {
                        version.to_string()
                    }
                };
            }
            return quote! { #field_ident: d.#field_ident.clone() };
        }

        match FieldKind::of(&field.ty) {
            FieldKind::Bool => quote! {
                #field_ident: config.get_bool(#option_id)
            },
            FieldKind::String => quote! {
                #field_ident: config.get_string(#option_id)
                    .unwrap_or_else(|| d.#field_ident.clone())
            },
            FieldKind::OptionEnum(inner_ty) => quote! {
                #field_ident: match config.get_enum(#option_id).as_deref() {
                    Some("none") => None,
                    Some(s) => <#inner_ty as std::str::FromStr>::from_str(s).ok(),
                    None => d.#field_ident.clone(),
                }
            },
            FieldKind::Enum => {
                let field_ty = &field.ty;
                quote! {
                    #field_ident: config.get_enum(#option_id)
                        .and_then(|s| <#field_ty as std::str::FromStr>::from_str(&s).ok())
                        .unwrap_or_else(|| d.#field_ident.clone())
                }
            }
            FieldKind::Other => quote! {
                #field_ident: d.#field_ident.clone()
            },
        }
    });

    quote! {
        pub fn from_config(config: &crate::editor::config::PresetConfig, version: &str) -> Self {
            let d = Self::default();
            Self {
                #(#field_assignments),*
            }
        }
    }
}

fn generate_to_preset_config(preset_id: &str, fields: &[PresetFieldOpts]) -> TokenStream {
    let set_statements = fields.iter().filter_map(|field| {
        let field_ident = field.ident.as_ref().unwrap();
        let option_id = field_ident.to_string();

        // Hidden fields are not exposed as editor options.
        if field.hidden {
            return None;
        }

        match FieldKind::of(&field.ty) {
            FieldKind::Bool => Some(quote! {
                config.set(#option_id.to_string(),
                    crate::editor::config::OptionValue::Bool(self.#field_ident));
            }),
            FieldKind::String => Some(quote! {
                config.set(#option_id.to_string(),
                    crate::editor::config::OptionValue::String(self.#field_ident.clone()));
            }),
            FieldKind::OptionEnum(inner_ty) => Some(quote! {
                config.set(#option_id.to_string(), crate::editor::config::OptionValue::Enum {
                    selected: self.#field_ident.as_ref()
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "none".to_string()),
                    variants: {
                        let mut v = vec!["none".to_string()];
                        v.extend(
                            <#inner_ty as strum::VariantNames>::VARIANTS
                                .iter()
                                .map(|s| s.to_string()),
                        );
                        v
                    },
                });
            }),
            FieldKind::Enum => {
                let field_ty = &field.ty;
                Some(quote! {
                    config.set(#option_id.to_string(), crate::editor::config::OptionValue::Enum {
                        selected: self.#field_ident.to_string(),
                        variants: <#field_ty as strum::VariantNames>::VARIANTS
                            .iter()
                            .map(|s| s.to_string())
                            .collect(),
                    });
                })
            }
            FieldKind::Other => None,
        }
    });

    quote! {
        pub fn to_preset_config(&self) -> crate::editor::config::PresetConfig {
            let mut config = crate::editor::config::PresetConfig::new(#preset_id.to_string());
            #(#set_statements)*
            config
        }
    }
}
