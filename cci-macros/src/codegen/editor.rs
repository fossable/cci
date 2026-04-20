use crate::preset::{PresetFieldOpts, PresetOpts};
use proc_macro2::TokenStream;
use quote::quote;

pub fn generate_editor_preset_impl(opts: &PresetOpts, fields: &[PresetFieldOpts]) -> TokenStream {
    let preset_ident = &opts.ident;
    let preset_id = opts.preset_id();
    let preset_name = opts.display_name();
    let preset_description = opts.doc_comment();
    let preset_category = &opts.category;

    let fields_impl = generate_fields_method(fields);
    let default_config_impl = generate_default_config_method(&preset_id, fields);
    let matches_project_impl = generate_matches_project_method(&opts.match_prefix());

    quote! {
        impl crate::editor::config::EditorPreset for #preset_ident {
            fn preset_id(&self) -> &'static str {
                #preset_id
            }

            fn preset_name(&self) -> &'static str {
                #preset_name
            }

            fn preset_description(&self) -> &'static str {
                #preset_description
            }

            fn preset_category(&self) -> &'static str {
                #preset_category
            }

            #fields_impl

            #default_config_impl

            #matches_project_impl

            fn generate(
                &self,
                config: &crate::editor::config::PresetConfig,
                platform: crate::editor::state::Platform,
                language_version: &str,
            ) -> crate::error::Result<String> {
                let preset = Self::from_config(config, language_version);
                crate::platforms::helpers::generate_for_platform(&preset, platform)
            }
        }
    }
}

fn generate_fields_method(fields: &[PresetFieldOpts]) -> TokenStream {
    let option_metas = fields.iter().filter(|field| !field.hidden).map(|field| {
        let field_ident = field.ident.as_ref().unwrap();
        let field_ty = &field.ty;

        let option_id = field_ident.to_string();

        let display_name = field
            .display
            .as_ref()
            .map(|s| s.as_str().to_string())
            .unwrap_or_else(|| option_id.clone());

        let description = field.doc_comment();

        // Determine default value based on type
        let default_value = if let Some(ref default_str) = field.default {
            let default_expr: TokenStream = default_str.parse().unwrap();
            match field_ty {
                syn::Type::Path(type_path) => {
                    let type_str = quote!(#type_path).to_string().replace(" ", "");
                    if type_str.starts_with("Option<") {
                        // Extract inner type from Option<T>
                        let inner_type_start = type_str.find('<').unwrap() + 1;
                        let inner_type_end = type_str.rfind('>').unwrap();
                        let inner_type_str = &type_str[inner_type_start..inner_type_end];
                        let inner_type = syn::parse_str::<syn::Type>(inner_type_str).unwrap();

                        // Option<EnumType> - include "none" variant
                        quote! {
                            crate::editor::config::OptionValue::Enum {
                                selected: {
                                    let opt_val: #field_ty = #default_expr;
                                    opt_val.as_ref().map(|v| v.as_str().to_string()).unwrap_or_else(|| "none".to_string())
                                },
                                variants: {
                                    let mut v = vec!["none".to_string()];
                                    v.extend(#inner_type::all_variants().iter().map(|s| s.to_string()));
                                    v
                                },
                            }
                        }
                    } else if type_str.contains("String") {
                        quote! {
                            crate::editor::config::OptionValue::String(#default_expr)
                        }
                    } else if type_str.contains("bool") {
                        quote! {
                            crate::editor::config::OptionValue::Bool(#default_expr)
                        }
                    } else {
                        // Enum
                        quote! {
                            crate::editor::config::OptionValue::Enum {
                                selected: (#default_expr).as_str().to_string(),
                                variants: #field_ty::all_variants().iter().map(|s| s.to_string()).collect(),
                            }
                        }
                    }
                }
                _ => quote! { crate::editor::config::OptionValue::Bool(false) },
            }
        } else {
            quote! { crate::editor::config::OptionValue::Bool(false) }
        };

        quote! {
            crate::editor::config::OptionMeta {
                id: #option_id.to_string(),
                display_name: #display_name.to_string(),
                description: #description.to_string(),
                default_value: #default_value,
                depends_on: None,
            }
        }
    });

    quote! {
        fn fields(&self) -> Vec<crate::editor::config::OptionMeta> {
            vec![
                #(#option_metas),*
            ]
        }
    }
}

fn generate_default_config_method(preset_id: &str, fields: &[PresetFieldOpts]) -> TokenStream {
    let set_statements = fields.iter().filter_map(|field| {
        // Skip hidden fields
        if field.hidden {
            return None;
        }

        let field_ty = &field.ty;
        let option_id = field.ident.as_ref().unwrap().to_string();

        let default_value = if let Some(ref default_str) = field.default {
            let default_expr: TokenStream = default_str.parse().unwrap();
            match field_ty {
                syn::Type::Path(type_path) => {
                    let type_str = quote!(#type_path).to_string().replace(" ", "");
                    if type_str.starts_with("Option<") {
                        // Extract inner type from Option<T>
                        let inner_type_start = type_str.find('<').unwrap() + 1;
                        let inner_type_end = type_str.rfind('>').unwrap();
                        let inner_type_str = &type_str[inner_type_start..inner_type_end];
                        let inner_type = syn::parse_str::<syn::Type>(inner_type_str).unwrap();

                        // Option<EnumType> - include "none" variant
                        quote! {
                            crate::editor::config::OptionValue::Enum {
                                selected: if detected {
                                    let opt_val: #field_ty = #default_expr;
                                    opt_val.as_ref().map(|v| v.as_str().to_string()).unwrap_or_else(|| "none".to_string())
                                } else {
                                    "none".to_string()
                                },
                                variants: {
                                    let mut v = vec!["none".to_string()];
                                    v.extend(#inner_type::all_variants().iter().map(|s| s.to_string()));
                                    v
                                },
                            }
                        }
                    } else if type_str.contains("String") {
                        quote! {
                            crate::editor::config::OptionValue::String(#default_expr)
                        }
                    } else if type_str.contains("bool") {
                        quote! {
                            crate::editor::config::OptionValue::Bool(if detected { #default_expr } else { false })
                        }
                    } else {
                        // Enum
                        quote! {
                            crate::editor::config::OptionValue::Enum {
                                selected: (#default_expr).as_str().to_string(),
                                variants: #field_ty::all_variants().iter().map(|s| s.to_string()).collect(),
                            }
                        }
                    }
                }
                _ => quote! { crate::editor::config::OptionValue::Bool(false) },
            }
        } else {
            quote! { crate::editor::config::OptionValue::Bool(false) }
        };

        Some(quote! {
            config.set(#option_id.to_string(), #default_value);
        })
    });

    quote! {
        fn default_config(&self, detected: bool) -> crate::editor::config::PresetConfig {
            let mut config = crate::editor::config::PresetConfig::new(#preset_id.to_string());
            #(#set_statements)*
            config
        }
    }
}

fn generate_matches_project_method(match_prefix: &str) -> TokenStream {
    quote! {
        fn matches_project(&self, project_type: &crate::detection::ProjectType, _working_dir: &std::path::Path) -> bool {
            project_type.to_string().starts_with(#match_prefix)
        }
    }
}
