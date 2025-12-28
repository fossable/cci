use proc_macro2::TokenStream;
use quote::{quote, format_ident};
use crate::preset::PresetFieldOpts;

pub fn generate_conversions(
    preset_ident: &syn::Ident,
    preset_id: &str,
    fields: &[PresetFieldOpts],
) -> TokenStream {
    let config_name = format_ident!("{}Config", preset_ident.to_string().strip_suffix("Preset").unwrap_or(&preset_ident.to_string()));

    // Generate from_config method (PresetConfig -> Preset instance)
    let from_config_impl = generate_from_config(preset_ident, fields);

    // Generate ron_to_preset_config (RON -> PresetConfig)
    let ron_to_preset_config = generate_ron_to_preset_config(preset_id, &config_name, fields);

    // Generate preset_config_to_ron (PresetConfig -> RON)
    let preset_config_to_ron = generate_preset_config_to_ron(&config_name, fields);

    quote! {
        impl #preset_ident {
            #from_config_impl
            #ron_to_preset_config
            #preset_config_to_ron
        }
    }
}

fn generate_from_config(
    _preset_ident: &syn::Ident,
    fields: &[PresetFieldOpts],
) -> TokenStream {
    let field_assignments = fields.iter().map(|field| {
        let field_ident = field.ident.as_ref().unwrap();
        let field_ty = &field.ty;

        // Determine the option ID
        let option_id = field.id.as_ref()
            .map(|s| s.clone())
            .unwrap_or_else(|| field_ident.to_string());

        // Check if this is the version field (hidden=true)
        if field.hidden {
            // Version comes from the version parameter
            quote! {
                #field_ident: version.to_string()
            }
        } else {
            // Check field type and generate appropriate getter
            match field_ty {
                syn::Type::Path(type_path) => {
                    let type_str = quote!(#type_path).to_string().replace(" ", "");
                    if type_str.starts_with("Vec") {
                        // Vec types use Default::default()
                        quote! {
                            #field_ident: Default::default()
                        }
                    } else if type_str.starts_with("Option<") {
                        // Extract inner type from Option<T>
                        let inner_type_start = type_str.find('<').unwrap() + 1;
                        let inner_type_end = type_str.rfind('>').unwrap();
                        let inner_type_str = &type_str[inner_type_start..inner_type_end];
                        let inner_type = syn::parse_str::<syn::Type>(inner_type_str).unwrap();

                        // For Option<EnumType>, use get_enum and map to from_str
                        quote! {
                            #field_ident: {
                                let value: #field_ty = match config.get_enum(#option_id).as_deref() {
                                    Some("none") => None,
                                    Some(s) => #inner_type::from_str(s),
                                    None => None,
                                };
                                value
                            }
                        }
                    } else if type_str == "String" {
                        quote! {
                            #field_ident: config.get_string(#option_id).unwrap_or_else(|| "".to_string())
                        }
                    } else if type_str == "bool" {
                        quote! {
                            #field_ident: config.get_bool(#option_id)
                        }
                    } else {
                        // Assume it's an enum with from_str method
                        quote! {
                            #field_ident: config.get_enum(#option_id)
                                .and_then(|s| #field_ty::from_str(&s))
                                .unwrap_or_default()
                        }
                    }
                }
                _ => {
                    quote! {
                        #field_ident: Default::default()
                    }
                }
            }
        }
    });

    quote! {
        pub fn from_config(config: &crate::editor::config::PresetConfig, version: &str) -> Self {
            Self {
                #(#field_assignments),*
            }
        }
    }
}

fn generate_ron_to_preset_config(
    preset_id: &str,
    config_name: &syn::Ident,
    fields: &[PresetFieldOpts],
) -> TokenStream {
    let set_statements = fields.iter().filter_map(|field| {
        let field_ident = field.ident.as_ref().unwrap();
        let field_ty = &field.ty;

        // Determine the option ID
        let option_id = field.id.as_ref()
            .map(|s| s.clone())
            .unwrap_or_else(|| field_ident.to_string());

        // Determine the RON field name
        let ron_field_name = if let Some(ref ron_field) = field.ron_field {
            format_ident!("{}", ron_field)
        } else if let Some(ref id) = field.id {
            let cleaned = id.strip_prefix("enable_").unwrap_or(id);
            format_ident!("{}", cleaned)
        } else {
            let field_str = field_ident.to_string();
            let cleaned = field_str.strip_prefix("enable_").unwrap_or(&field_str);
            format_ident!("{}", cleaned)
        };

        // Skip hidden fields (they're not in PresetConfig, handled separately)
        if field.hidden {
            return None;
        }

        match field_ty {
            syn::Type::Path(type_path) => {
                let type_str = quote!(#type_path).to_string().replace(" ", "");
                if type_str.starts_with("Vec") {
                    // Skip Vec types - they're not exposed in PresetConfig
                    None
                } else if type_str.starts_with("Option<") {
                    // Extract inner type from Option<T>
                    // We need to get the inner enum type to call all_variants()
                    // Parse the inner type name from "Option < EnumType >"
                    let inner_type_start = type_str.find('<').unwrap() + 1;
                    let inner_type_end = type_str.rfind('>').unwrap();
                    let inner_type_str = &type_str[inner_type_start..inner_type_end];
                    let inner_type = syn::parse_str::<syn::Type>(inner_type_str).unwrap();

                    Some(quote! {
                        if let Some(ref value) = ron.#ron_field_name {
                            config.set(#option_id.to_string(), crate::editor::config::OptionValue::Enum {
                                selected: value.as_str().to_string(),
                                variants: #inner_type::all_variants().iter().map(|s| s.to_string()).collect(),
                            });
                        } else {
                            // Set with empty selection for None
                            config.set(#option_id.to_string(), crate::editor::config::OptionValue::Enum {
                                selected: "none".to_string(),
                                variants: {
                                    let mut v = vec!["none".to_string()];
                                    v.extend(#inner_type::all_variants().iter().map(|s| s.to_string()));
                                    v
                                },
                            });
                        }
                    })
                } else if type_str == "String" {
                    Some(quote! {
                        config.set(#option_id.to_string(), crate::editor::config::OptionValue::String(ron.#ron_field_name.clone()));
                    })
                } else if type_str == "bool" {
                    Some(quote! {
                        config.set(#option_id.to_string(), crate::editor::config::OptionValue::Bool(ron.#ron_field_name));
                    })
                } else {
                    // Enum type
                    Some(quote! {
                        config.set(#option_id.to_string(), crate::editor::config::OptionValue::Enum {
                            selected: ron.#ron_field_name.as_str().to_string(),
                            variants: #field_ty::all_variants().iter().map(|s| s.to_string()).collect(),
                        });
                    })
                }
            }
            _ => None,
        }
    });

    quote! {
        pub fn ron_to_preset_config(ron: #config_name) -> crate::editor::config::PresetConfig {
            let mut config = crate::editor::config::PresetConfig::new(#preset_id.to_string());
            #(#set_statements)*
            config
        }
    }
}

fn generate_preset_config_to_ron(
    config_name: &syn::Ident,
    fields: &[PresetFieldOpts],
) -> TokenStream {
    let field_assignments = fields.iter().map(|field| {
        let field_ident = field.ident.as_ref().unwrap();
        let field_ty = &field.ty;

        // Determine the option ID
        let option_id = field.id.as_ref()
            .map(|s| s.clone())
            .unwrap_or_else(|| field_ident.to_string());

        // Determine the RON field name
        let ron_field_name = if let Some(ref ron_field) = field.ron_field {
            format_ident!("{}", ron_field)
        } else if let Some(ref id) = field.id {
            let cleaned = id.strip_prefix("enable_").unwrap_or(id);
            format_ident!("{}", cleaned)
        } else {
            let field_str = field_ident.to_string();
            let cleaned = field_str.strip_prefix("enable_").unwrap_or(&field_str);
            format_ident!("{}", cleaned)
        };

        // Get default value
        let default_val = field.default.as_ref()
            .map(|s| s.parse::<TokenStream>().unwrap())
            .unwrap_or_else(|| quote! { Default::default() });

        if field.hidden {
            // Version field - use default or from config
            match field_ty {
                syn::Type::Path(type_path) => {
                    let type_str = quote!(#type_path).to_string();
                    if type_str.contains("String") {
                        quote! {
                            #ron_field_name: config.get_string(#option_id).unwrap_or_else(|| #default_val)
                        }
                    } else {
                        quote! {
                            #ron_field_name: #default_val
                        }
                    }
                }
                _ => quote! { #ron_field_name: #default_val },
            }
        } else {
            match field_ty {
                syn::Type::Path(type_path) => {
                    let type_str = quote!(#type_path).to_string().replace(" ", "");
                    if type_str.starts_with("Vec") {
                        // Vec types use default value
                        quote! {
                            #ron_field_name: #default_val
                        }
                    } else if type_str.starts_with("Option<") {
                        // Extract inner type from Option<T>
                        let inner_type_start = type_str.find('<').unwrap() + 1;
                        let inner_type_end = type_str.rfind('>').unwrap();
                        let inner_type_str = &type_str[inner_type_start..inner_type_end];
                        let inner_type = syn::parse_str::<syn::Type>(inner_type_str).unwrap();

                        // Option<EnumType>
                        quote! {
                            #ron_field_name: match config.get_enum(#option_id).as_deref() {
                                Some("none") => None,
                                Some(s) => #inner_type::from_str(s),
                                None => #default_val,
                            }
                        }
                    } else if type_str == "String" {
                        quote! {
                            #ron_field_name: config.get_string(#option_id).unwrap_or_else(|| #default_val)
                        }
                    } else if type_str == "bool" {
                        quote! {
                            #ron_field_name: config.get_bool(#option_id)
                        }
                    } else {
                        // Enum
                        quote! {
                            #ron_field_name: config.get_enum(#option_id)
                                .and_then(|s| #field_ty::from_str(&s))
                                .unwrap_or_else(|| #default_val)
                        }
                    }
                }
                _ => quote! { #ron_field_name: #default_val },
            }
        }
    });

    quote! {
        pub fn preset_config_to_ron(config: &crate::editor::config::PresetConfig) -> #config_name {
            #config_name {
                #(#field_assignments),*
            }
        }
    }
}
